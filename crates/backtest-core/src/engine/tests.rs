use crate::result::ScenarioCode;
use crate::{BacktestConfig, DatasetMetadata, DomainError, IvScenario};

use super::accounting::{checked_product, classify_margin, state_with_code, time_years};
use super::{MINUTE_NS, WINDOW_MINUTES, run_backtest};

fn test_config() -> BacktestConfig {
    BacktestConfig {
        initial_capital_usd: 1_000.0,
        base_iv: 0.5,
        risk_free_rate: 0.01,
        carry_rate: 0.02,
        margin_per_straddle_usd: 100.0,
        quantity_step: 0.1,
    }
}

fn test_dataset() -> DatasetMetadata {
    DatasetMetadata {
        dataset_id: "engine-unit".into(),
        source: "synthetic".into(),
        symbol: "BTCUSDT-SWAP".into(),
        interval_seconds: 60,
        timezone: "UTC".into(),
    }
}

#[test]
fn unit_target_exercises_the_complete_engine_surface() {
    let timestamps: Vec<_> = (0..=WINDOW_MINUTES)
        .map(|index| index as i64 * MINUTE_NS)
        .collect();
    let close = vec![100.0; timestamps.len()];
    let result = run_backtest(
        &timestamps,
        &close,
        test_dataset(),
        test_config(),
        &[
            IvScenario::Baseline,
            IvScenario::Stress2x { after_minutes: 1 },
            IvScenario::Stress3x { after_minutes: 1 },
        ],
    )
    .unwrap();
    assert_eq!(result.summary.completed_trade_count, [1, 1, 1]);
    assert_eq!(result.reserve_attempts.outcome.len(), 2);
}

#[test]
fn checked_arithmetic_reports_nonfinite_results() {
    assert_eq!(
        checked_product(f64::MAX, 2.0, "product"),
        Err(DomainError::NumericOverflow { field: "product" })
    );
    assert_eq!(checked_product(2.0, 3.0, "product").unwrap(), 6.0);
}

#[test]
fn time_to_expiry_rejects_overflow_and_negative_duration() {
    assert_eq!(
        time_years(i64::MIN, i64::MAX),
        Err(DomainError::NumericOverflow {
            field: "time_to_expiry"
        })
    );
    assert_eq!(
        time_years(1, 0),
        Err(DomainError::NumericOverflow {
            field: "time_to_expiry"
        })
    );
    assert_eq!(time_years(0, 0).unwrap(), 0.0);
}

#[test]
fn state_accounting_rejects_nonfinite_equity_and_available_margin() {
    let mut state = state_with_code(ScenarioCode::Baseline, 1.0);
    state.cash = f64::MAX;
    state.liability = -f64::MAX;
    assert_eq!(
        classify_margin(&mut state),
        Err(DomainError::NumericOverflow { field: "equity" })
    );

    state.cash = -f64::MAX;
    state.liability = 0.0;
    state.locked_margin = f64::MAX;
    assert_eq!(
        classify_margin(&mut state),
        Err(DomainError::NumericOverflow {
            field: "available_margin"
        })
    );
}

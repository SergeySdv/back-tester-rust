use crate::support;

use backtest_core::{
    DomainError, IvScenario, ReserveOutcome, ReserveReason, TerminalStatus, floor_steps,
    money_tolerance, reserve_triggered, run_backtest,
};
use support::{config, dataset, flat, linear, timestamps};

#[test]
fn decimal_step_boundaries_use_the_exact_normalized_floor_algorithm() {
    let on = 1.0_f64;
    let immediately_below = f64::from_bits(on.to_bits() - 1);
    let immediately_above = f64::from_bits(on.to_bits() + 1);
    for budget in [immediately_below, on, immediately_above] {
        let steps = floor_steps(budget, 10.0, 0.1).unwrap();
        assert_eq!(steps, 1);
        let step_margin = steps as f64 * 10.0 * 0.1;
        assert!(step_margin <= budget + money_tolerance(budget, step_margin));
    }
    let outside_tolerance = 1.0 - 32.0 * f64::EPSILON;
    assert_eq!(floor_steps(outside_tolerance, 10.0, 0.1).unwrap(), 0);
}

#[test]
fn sizing_rejects_nonfinite_products_ratios_and_u64_overflow() {
    assert!(matches!(
        floor_steps(-1.0, 1.0, 1.0),
        Err(DomainError::NumericOverflow { field: "budget" })
    ));
    assert!(matches!(
        floor_steps(f64::NAN, 1.0, 1.0),
        Err(DomainError::NumericOverflow { field: "budget" })
    ));
    assert!(matches!(
        floor_steps(1.0, f64::MAX, 2.0),
        Err(DomainError::NumericOverflow {
            field: "margin_per_step"
        })
    ));
    assert!(matches!(
        floor_steps(f64::MAX, f64::MIN_POSITIVE, f64::MIN_POSITIVE),
        Err(DomainError::NumericOverflow { .. })
    ));
    assert!(matches!(
        floor_steps(u64::MAX as f64, 1.0, 1.0),
        Err(DomainError::NumericOverflow {
            field: "quantity_steps"
        })
    ));
}

#[test]
fn reserve_trigger_is_inclusive_at_exactly_one_and_a_half_times_entry_iv() {
    let entry_iv = 0.4_f64;
    let threshold = 1.5 * entry_iv;
    assert!(!reserve_triggered(
        f64::from_bits(threshold.to_bits() - 1),
        entry_iv
    ));
    assert!(reserve_triggered(threshold, entry_iv));
    assert!(reserve_triggered(
        f64::from_bits(threshold.to_bits() + 1),
        entry_iv
    ));
}

#[test]
fn zero_first_entry_is_typed_error_but_later_zero_entry_is_terminal_result() {
    let mut insufficient = config();
    insufficient.initial_capital_usd = 1.0;
    assert_eq!(
        run_backtest(
            &timestamps(1_441),
            &flat(1_441, 100.0),
            dataset(),
            insufficient,
            &[IvScenario::Baseline]
        ),
        Err(DomainError::InsufficientInitialCapital)
    );

    let mut exhausted_config = config();
    exhausted_config.initial_capital_usd = 100.0;
    let mut close = linear(1_441, 100.0, 230.0);
    close.extend(flat(1_440, 230.0));
    let result = run_backtest(
        &timestamps(2_881),
        &close,
        dataset(),
        exhausted_config,
        &[IvScenario::Baseline],
    )
    .unwrap();
    assert_eq!(
        result.summary.terminal_status,
        [TerminalStatus::CapitalExhausted]
    );
    assert_eq!(result.summary.completed_trade_count, [1]);
    assert_eq!(result.summary.processed_row_count, [1_441]);
    assert_eq!(result.summary.ignored_input_row_count, [1_440]);
    assert_eq!(result.summary.skipped_incomplete_window_count, [0]);
    assert!(!result.equity.active_trade_id_valid[1_440]);

    let mut negative_close = linear(1_441, 100.0, 1_000.0);
    negative_close.extend(flat(1_440, 1_000.0));
    let negative = run_backtest(
        &timestamps(2_881),
        &negative_close,
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();
    assert_eq!(
        negative.summary.terminal_status,
        [TerminalStatus::CapitalExhausted]
    );
    assert!(negative.summary.final_equity[0] < 0.0);
}

#[test]
fn triggered_zero_reserve_is_logged_below_step_without_a_tranche() {
    let mut tiny = config();
    tiny.initial_capital_usd = 20.0;
    let result = run_backtest(
        &timestamps(1_441),
        &flat(1_441, 10.0),
        dataset(),
        tiny,
        &[IvScenario::Stress2x {
            after_minutes: 1_439,
        }],
    )
    .unwrap();
    assert_eq!(result.reserve_attempts.outcome, [ReserveOutcome::Rejected]);
    assert_eq!(
        result.reserve_attempts.reason,
        [ReserveReason::BelowQuantityStep]
    );
    assert_eq!(result.reserve_attempts.requested_quantity_steps, [0]);
    assert_eq!(result.executed_tranches.tranche_kind.len(), 1);
}

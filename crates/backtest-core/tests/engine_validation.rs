use crate::support;

use backtest_core::{DomainError, InvalidValue, IvScenario, run_backtest};
use support::{MINUTE_NS, config, dataset, flat, timestamps};

#[test]
fn mandatory_history_boundaries_1440_1441_1442_2880_2881_are_exact() {
    let short = run_backtest(
        &timestamps(1_440),
        &flat(1_440, 100.0),
        dataset(),
        config(),
        &[IvScenario::Baseline],
    );
    assert_eq!(
        short,
        Err(DomainError::InsufficientHistory {
            minimum: 1_441,
            actual: 1_440
        })
    );

    for (points, processed, ignored, skipped, trades) in [
        (1_441, 1_441, 0, 0, 1),
        (1_442, 1_441, 1, 1, 1),
        (2_880, 1_441, 1_439, 1, 1),
        (2_881, 2_881, 0, 0, 2),
    ] {
        let result = run_backtest(
            &timestamps(points),
            &flat(points, 100.0),
            dataset(),
            config(),
            &[IvScenario::Baseline],
        )
        .unwrap();
        assert_eq!(result.summary.processed_row_count, [processed]);
        assert_eq!(result.summary.ignored_input_row_count, [ignored]);
        assert_eq!(result.summary.skipped_incomplete_window_count, [skipped]);
        assert_eq!(result.summary.completed_trade_count, [trades]);
        assert_eq!(result.equity.timestamp_ns.len(), processed as usize);
    }
}

#[test]
fn invalid_arrays_and_prices_fail_without_a_result() {
    let valid_timestamps = timestamps(1_441);
    let valid_close = flat(1_441, 100.0);
    assert!(matches!(
        run_backtest(
            &valid_timestamps,
            &valid_close[..1_440],
            dataset(),
            config(),
            &[IvScenario::Baseline]
        ),
        Err(DomainError::LengthMismatch { field: "close", .. })
    ));

    for (index, replacement) in [(10, 0.0), (11, -1.0), (12, f64::NAN), (13, f64::INFINITY)] {
        let mut close = valid_close.clone();
        close[index] = replacement;
        assert!(matches!(
            run_backtest(
                &valid_timestamps,
                &close,
                dataset(),
                config(),
                &[IvScenario::Baseline]
            ),
            Err(DomainError::InvalidPrice { index: actual, .. }) if actual == index
        ));
    }
}

#[test]
fn gaps_duplicates_disorder_and_timestamp_overflow_are_rejected() {
    for (index, replacement) in [
        (10, 9 * MINUTE_NS),
        (10, 8 * MINUTE_NS),
        (10, 11 * MINUTE_NS),
    ] {
        let mut time = timestamps(1_441);
        time[index] = replacement;
        assert!(matches!(
            run_backtest(
                &time,
                &flat(1_441, 100.0),
                dataset(),
                config(),
                &[IvScenario::Baseline]
            ),
            Err(DomainError::InvalidTimestamp { index: actual, .. }) if actual == index
        ));
    }

    let mut overflow = timestamps(1_441);
    overflow[0] = i64::MIN;
    assert!(matches!(
        run_backtest(
            &overflow,
            &flat(1_441, 100.0),
            dataset(),
            config(),
            &[IvScenario::Baseline]
        ),
        Err(DomainError::InvalidTimestamp { index: 1, .. })
    ));
}

#[test]
fn invalid_metadata_and_config_are_rejected_before_execution() {
    let mut invalid_dataset = dataset();
    invalid_dataset.timezone = "Europe/Moscow".into();
    assert_eq!(
        run_backtest(
            &timestamps(1_441),
            &flat(1_441, 100.0),
            invalid_dataset,
            config(),
            &[IvScenario::Baseline]
        ),
        Err(DomainError::InvalidField {
            field: "dataset.timezone",
            reason: InvalidValue::Unsupported
        })
    );

    let mut overflowing = config();
    overflowing.initial_capital_usd = f64::MAX;
    overflowing.margin_per_straddle_usd = f64::MAX;
    overflowing.quantity_step = 2.0;
    assert_eq!(
        run_backtest(
            &timestamps(1_441),
            &flat(1_441, 100.0),
            dataset(),
            overflowing,
            &[IvScenario::Baseline]
        ),
        Err(DomainError::NumericOverflow {
            field: "margin_per_step"
        })
    );
}

#[test]
fn new_engine_errors_are_typed_and_actionable() {
    let errors = [
        DomainError::InsufficientHistory {
            minimum: 1_441,
            actual: 12,
        },
        DomainError::InvalidTimestamp {
            index: 7,
            previous_ns: 60,
            current_ns: 61,
        },
        DomainError::InvalidPrice {
            index: 8,
            reason: InvalidValue::NotFinite,
        },
        DomainError::InsufficientInitialCapital,
        DomainError::NumericOverflow { field: "quantity" },
    ];
    for error in errors {
        let message = error.to_string();
        assert!(!message.is_empty());
        assert!(!message.contains("unknown"));
    }
}

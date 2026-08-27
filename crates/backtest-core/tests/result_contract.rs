use crate::support;

use backtest_core::{
    COMPLETED_TRADES_SCHEMA, EQUITY_SCHEMA, EXECUTED_TRANCHES_SCHEMA, IvScenario,
    RESERVE_ATTEMPTS_SCHEMA, ReserveOutcome, ReserveReason, SUMMARY_SCHEMA, ScenarioCode,
    TerminalStatus, TrancheKind, run_backtest,
};
use support::{config, dataset, flat, timestamps};

fn names(schema: &[backtest_core::ColumnSchema]) -> Vec<&str> {
    schema.iter().map(|column| column.name).collect()
}

fn dtypes(schema: &[backtest_core::ColumnSchema]) -> Vec<&str> {
    schema.iter().map(|column| column.logical_dtype).collect()
}

#[test]
fn public_table_column_names_order_dtypes_and_nullability_match_section_10() {
    assert_eq!(
        names(EQUITY_SCHEMA),
        [
            "timestamp_ns",
            "scenario_id",
            "spot",
            "active_trade_id",
            "active_iv",
            "cash",
            "option_liability",
            "locked_margin",
            "available_margin",
            "equity",
            "pnl",
            "running_peak",
            "drawdown_usd",
            "drawdown_pct",
            "margin_breached"
        ]
    );
    assert_eq!(
        EQUITY_SCHEMA
            .iter()
            .filter(|column| column.nullable)
            .map(|column| column.name)
            .collect::<Vec<_>>(),
        ["active_trade_id", "active_iv"]
    );
    assert_eq!(EQUITY_SCHEMA[0].logical_dtype, "int64");
    assert_eq!(EQUITY_SCHEMA[3].logical_dtype, "uint64");
    assert_eq!(EQUITY_SCHEMA[14].logical_dtype, "bool");
    assert_eq!(
        EQUITY_SCHEMA
            .iter()
            .map(|column| column.logical_dtype)
            .collect::<Vec<_>>(),
        [
            "int64", "string", "float64", "uint64", "float64", "float64", "float64", "float64",
            "float64", "float64", "float64", "float64", "float64", "float64", "bool"
        ]
    );
    assert_eq!(
        names(COMPLETED_TRADES_SCHEMA),
        [
            "scenario_id",
            "trade_id",
            "entry_timestamp_ns",
            "expiry_timestamp_ns",
            "strike",
            "entry_equity",
            "initial_quantity_steps",
            "initial_quantity",
            "reserve_attempted",
            "reserve_executed_quantity_steps",
            "reserve_executed_quantity",
            "total_received_premium",
            "settlement_spot",
            "total_expiry_payoff",
            "realized_pnl",
            "margin_breached_during_trade"
        ]
    );
    assert_eq!(
        names(EXECUTED_TRANCHES_SCHEMA),
        [
            "scenario_id",
            "trade_id",
            "tranche_id",
            "tranche_kind",
            "execution_timestamp_ns",
            "expiry_timestamp_ns",
            "strike",
            "quantity_steps",
            "quantity",
            "active_iv",
            "call_premium_per_unit",
            "put_premium_per_unit",
            "total_premium_per_unit",
            "received_premium",
            "locked_margin"
        ]
    );
    assert_eq!(
        names(RESERVE_ATTEMPTS_SCHEMA),
        [
            "scenario_id",
            "trade_id",
            "attempt_timestamp_ns",
            "requested_quantity_steps",
            "executed_quantity_steps",
            "requested_quantity",
            "executed_quantity",
            "available_margin_before",
            "reserve_budget_remaining",
            "outcome",
            "reason"
        ]
    );
    assert_eq!(
        names(SUMMARY_SCHEMA),
        [
            "scenario_id",
            "terminal_status",
            "terminal_timestamp_ns",
            "initial_equity",
            "final_equity",
            "total_pnl",
            "return_pct",
            "maximum_drawdown_usd",
            "maximum_drawdown_pct",
            "minimum_equity",
            "minimum_available_margin",
            "maximum_locked_margin",
            "completed_trade_count",
            "reserve_attempt_count",
            "full_reserve_count",
            "reduced_reserve_count",
            "rejected_reserve_count",
            "skipped_incomplete_window_count",
            "input_row_count",
            "processed_row_count",
            "ignored_input_row_count",
            "any_margin_breach"
        ]
    );
    assert!(
        COMPLETED_TRADES_SCHEMA
            .iter()
            .all(|column| !column.nullable)
    );
    assert!(
        EXECUTED_TRANCHES_SCHEMA
            .iter()
            .all(|column| !column.nullable)
    );
    assert!(
        RESERVE_ATTEMPTS_SCHEMA
            .iter()
            .all(|column| !column.nullable)
    );
    assert!(SUMMARY_SCHEMA.iter().all(|column| !column.nullable));
    assert_eq!(
        dtypes(COMPLETED_TRADES_SCHEMA),
        [
            "string", "uint64", "int64", "int64", "float64", "float64", "uint64", "float64",
            "bool", "uint64", "float64", "float64", "float64", "float64", "float64", "bool"
        ]
    );
    assert_eq!(
        dtypes(EXECUTED_TRANCHES_SCHEMA),
        [
            "string", "uint64", "uint64", "string", "int64", "int64", "float64", "uint64",
            "float64", "float64", "float64", "float64", "float64", "float64", "float64"
        ]
    );
    assert_eq!(
        dtypes(RESERVE_ATTEMPTS_SCHEMA),
        [
            "string", "uint64", "int64", "uint64", "uint64", "float64", "float64", "float64",
            "float64", "string", "string"
        ]
    );
    assert_eq!(
        dtypes(SUMMARY_SCHEMA),
        [
            "string", "string", "int64", "float64", "float64", "float64", "float64", "float64",
            "float64", "float64", "float64", "float64", "uint64", "uint64", "uint64", "uint64",
            "uint64", "uint8", "uint64", "uint64", "uint64", "bool"
        ]
    );
}

#[test]
fn closed_enum_codes_expose_only_the_documented_string_values() {
    assert_eq!(ScenarioCode::Baseline.as_str(), "baseline");
    assert_eq!(ScenarioCode::Stress2x.as_str(), "stress_2x");
    assert_eq!(ScenarioCode::Stress3x.as_str(), "stress_3x");
    assert_eq!(TrancheKind::Initial.as_str(), "initial");
    assert_eq!(TrancheKind::Reserve.as_str(), "reserve");
    assert_eq!(ReserveOutcome::Full.as_str(), "full");
    assert_eq!(ReserveOutcome::Reduced.as_str(), "reduced");
    assert_eq!(ReserveOutcome::Rejected.as_str(), "rejected");
    assert_eq!(ReserveReason::None.as_str(), "none");
    assert_eq!(
        ReserveReason::LimitedByAvailableMargin.as_str(),
        "limited_by_available_margin"
    );
    assert_eq!(
        ReserveReason::BelowQuantityStep.as_str(),
        "below_quantity_step"
    );
    assert_eq!(
        ReserveReason::NoAvailableMargin.as_str(),
        "no_available_margin"
    );
    assert_eq!(TerminalStatus::Completed.as_str(), "completed");
    assert_eq!(
        TerminalStatus::CapitalExhausted.as_str(),
        "capital_exhausted"
    );
}

#[test]
fn buffers_have_equal_lengths_validity_masks_and_canonical_row_order() {
    let result = run_backtest(
        &timestamps(2_881),
        &flat(2_881, 100.0),
        dataset(),
        config(),
        &[
            IvScenario::Stress3x { after_minutes: 720 },
            IvScenario::Baseline,
            IvScenario::Stress2x { after_minutes: 720 },
        ],
    )
    .unwrap();
    let equity_len = result.equity.timestamp_ns.len();
    for length in [
        result.equity.scenario_id.len(),
        result.equity.spot.len(),
        result.equity.active_trade_id.len(),
        result.equity.active_trade_id_valid.len(),
        result.equity.active_iv.len(),
        result.equity.active_iv_valid.len(),
        result.equity.cash.len(),
        result.equity.option_liability.len(),
        result.equity.locked_margin.len(),
        result.equity.available_margin.len(),
        result.equity.equity.len(),
        result.equity.pnl.len(),
        result.equity.running_peak.len(),
        result.equity.drawdown_usd.len(),
        result.equity.drawdown_pct.len(),
        result.equity.margin_breached.len(),
    ] {
        assert_eq!(length, equity_len);
    }
    assert_eq!(equity_len, 3 * 2_881);
    assert_eq!(
        result.summary.scenario_id,
        [
            ScenarioCode::Baseline,
            ScenarioCode::Stress2x,
            ScenarioCode::Stress3x
        ]
    );
    assert_eq!(result.completed_trades.trade_id, [0, 1, 0, 1, 0, 1]);
    assert!(
        result
            .equity
            .active_trade_id_valid
            .iter()
            .filter(|valid| !**valid)
            .count()
            == 3
    );
    assert!(
        result
            .equity
            .active_iv_valid
            .iter()
            .filter(|valid| !**valid)
            .count()
            == 3
    );
    assert!(
        result
            .equity
            .active_iv
            .iter()
            .all(|value| value.is_finite())
    );

    let trades_len = result.completed_trades.trade_id.len();
    for length in [
        result.completed_trades.scenario_id.len(),
        result.completed_trades.entry_timestamp_ns.len(),
        result.completed_trades.expiry_timestamp_ns.len(),
        result.completed_trades.strike.len(),
        result.completed_trades.entry_equity.len(),
        result.completed_trades.initial_quantity_steps.len(),
        result.completed_trades.initial_quantity.len(),
        result.completed_trades.reserve_attempted.len(),
        result
            .completed_trades
            .reserve_executed_quantity_steps
            .len(),
        result.completed_trades.reserve_executed_quantity.len(),
        result.completed_trades.total_received_premium.len(),
        result.completed_trades.settlement_spot.len(),
        result.completed_trades.total_expiry_payoff.len(),
        result.completed_trades.realized_pnl.len(),
        result.completed_trades.margin_breached_during_trade.len(),
    ] {
        assert_eq!(length, trades_len);
    }
    let tranches_len = result.executed_tranches.trade_id.len();
    for length in [
        result.executed_tranches.scenario_id.len(),
        result.executed_tranches.tranche_id.len(),
        result.executed_tranches.tranche_kind.len(),
        result.executed_tranches.execution_timestamp_ns.len(),
        result.executed_tranches.expiry_timestamp_ns.len(),
        result.executed_tranches.strike.len(),
        result.executed_tranches.quantity_steps.len(),
        result.executed_tranches.quantity.len(),
        result.executed_tranches.active_iv.len(),
        result.executed_tranches.call_premium_per_unit.len(),
        result.executed_tranches.put_premium_per_unit.len(),
        result.executed_tranches.total_premium_per_unit.len(),
        result.executed_tranches.received_premium.len(),
        result.executed_tranches.locked_margin.len(),
    ] {
        assert_eq!(length, tranches_len);
    }
    let attempts_len = result.reserve_attempts.trade_id.len();
    for length in [
        result.reserve_attempts.scenario_id.len(),
        result.reserve_attempts.attempt_timestamp_ns.len(),
        result.reserve_attempts.requested_quantity_steps.len(),
        result.reserve_attempts.executed_quantity_steps.len(),
        result.reserve_attempts.requested_quantity.len(),
        result.reserve_attempts.executed_quantity.len(),
        result.reserve_attempts.available_margin_before.len(),
        result.reserve_attempts.reserve_budget_remaining.len(),
        result.reserve_attempts.outcome.len(),
        result.reserve_attempts.reason.len(),
    ] {
        assert_eq!(length, attempts_len);
    }
    let summary_len = result.summary.scenario_id.len();
    for length in [
        result.summary.terminal_status.len(),
        result.summary.terminal_timestamp_ns.len(),
        result.summary.initial_equity.len(),
        result.summary.final_equity.len(),
        result.summary.total_pnl.len(),
        result.summary.return_pct.len(),
        result.summary.maximum_drawdown_usd.len(),
        result.summary.maximum_drawdown_pct.len(),
        result.summary.minimum_equity.len(),
        result.summary.minimum_available_margin.len(),
        result.summary.maximum_locked_margin.len(),
        result.summary.completed_trade_count.len(),
        result.summary.reserve_attempt_count.len(),
        result.summary.full_reserve_count.len(),
        result.summary.reduced_reserve_count.len(),
        result.summary.rejected_reserve_count.len(),
        result.summary.skipped_incomplete_window_count.len(),
        result.summary.input_row_count.len(),
        result.summary.processed_row_count.len(),
        result.summary.ignored_input_row_count.len(),
        result.summary.any_margin_breach.len(),
    ] {
        assert_eq!(length, summary_len);
    }
    assert_eq!(summary_len, 3);
}

#[test]
fn run_metadata_preserves_dataset_config_scenarios_and_honest_model_label() {
    let dataset = dataset();
    let config = config();
    let result = run_backtest(
        &timestamps(1_441),
        &flat(1_441, 100.0),
        dataset.clone(),
        config,
        &[
            IvScenario::Stress2x { after_minutes: 33 },
            IvScenario::Baseline,
        ],
    )
    .unwrap();
    assert_eq!(result.metadata.dataset, dataset);
    assert_eq!(result.metadata.config, config);
    assert_eq!(result.metadata.initial_allocation, 0.70);
    assert_eq!(result.metadata.reserve_allocation, 0.30);
    assert_eq!(result.metadata.reserve_trigger_multiple, 1.50);
    assert_eq!(result.metadata.seconds_per_year, 31_536_000);
    assert_eq!(
        result.metadata.pricing_model,
        "synthetic Black-Scholes scenario backtest"
    );
    assert_eq!(
        result.metadata.scenarios[0].scenario_id,
        ScenarioCode::Baseline
    );
    assert_eq!(result.metadata.scenarios[0].shock_after_minutes, None);
    assert_eq!(
        result.metadata.scenarios[1].scenario_id,
        ScenarioCode::Stress2x
    );
    assert_eq!(result.metadata.scenarios[1].multiplier, 2.0);
    assert_eq!(result.metadata.scenarios[1].shock_after_minutes, Some(33));
}

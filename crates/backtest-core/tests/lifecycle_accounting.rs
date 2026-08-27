use crate::support;

use backtest_core::{
    IvScenario, ReserveOutcome, ScenarioCode, TerminalStatus, money_tolerance, run_backtest,
};
use support::{config, dataset, flat, linear, timestamps};

#[test]
fn initial_entry_uses_only_current_close_and_expiry_is_exactly_24_elapsed_hours() {
    let time = timestamps(1_441);
    let flat_result = run_backtest(
        &time,
        &flat(1_441, 100.0),
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();
    let mut different_future = flat(1_441, 1_000.0);
    different_future[0] = 100.0;
    let changed_result = run_backtest(
        &time,
        &different_future,
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();
    assert_eq!(flat_result.executed_tranches.strike[0], 100.0);
    assert_eq!(changed_result.executed_tranches.strike[0], 100.0);
    assert_eq!(
        flat_result.executed_tranches.received_premium[0].to_bits(),
        changed_result.executed_tranches.received_premium[0].to_bits()
    );
    assert_eq!(
        flat_result.executed_tranches.quantity_steps[0],
        changed_result.executed_tranches.quantity_steps[0]
    );
    assert_eq!(
        flat_result.executed_tranches.expiry_timestamp_ns[0]
            - flat_result.executed_tranches.execution_timestamp_ns[0],
        24 * 60 * 60 * 1_000_000_000
    );
}

#[test]
fn two_day_boundary_settles_then_reenters_and_emits_one_post_event_row() {
    let time = timestamps(2_881);
    let result = run_backtest(
        &time,
        &flat(2_881, 100.0),
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();
    assert_eq!(result.completed_trades.trade_id, [0, 1]);
    assert_eq!(
        result.completed_trades.entry_timestamp_ns,
        [time[0], time[1_440]]
    );
    assert_eq!(
        result.completed_trades.expiry_timestamp_ns,
        [time[1_440], time[2_880]]
    );
    assert_eq!(result.completed_trades.strike, [100.0, 100.0]);
    assert_eq!(
        result
            .equity
            .timestamp_ns
            .iter()
            .filter(|timestamp| **timestamp == time[1_440])
            .count(),
        1
    );
    assert_eq!(result.equity.active_trade_id[1_440], 1);
    assert!(result.equity.active_trade_id_valid[1_440]);
    assert_eq!(result.equity.active_iv[1_440], config().base_iv);
    assert_eq!(
        result.completed_trades.margin_breached_during_trade,
        [false, false]
    );
    let boundary_equity = result.equity.equity[1_440];
    let second_entry_equity = result.completed_trades.entry_equity[1];
    assert!(
        (boundary_equity - second_entry_equity).abs()
            <= money_tolerance(boundary_equity, second_entry_equity)
    );
    assert!(!result.equity.active_trade_id_valid[2_880]);
    assert!(!result.equity.active_iv_valid[2_880]);
}

#[test]
fn flat_rising_and_falling_paths_reconcile_cash_liability_margin_and_trade_pnl() {
    for close in [
        flat(1_441, 100.0),
        linear(1_441, 100.0, 120.0),
        linear(1_441, 100.0, 80.0),
    ] {
        let initial = config().initial_capital_usd;
        let result = run_backtest(
            &timestamps(1_441),
            &close,
            dataset(),
            config(),
            &[IvScenario::Baseline],
        )
        .unwrap();
        for index in 0..result.equity.equity.len() {
            assert_eq!(
                result.equity.equity[index],
                result.equity.cash[index] - result.equity.option_liability[index]
            );
            assert_eq!(
                result.equity.available_margin[index],
                result.equity.equity[index] - result.equity.locked_margin[index]
            );
            assert_eq!(
                result.equity.pnl[index],
                result.equity.equity[index] - initial
            );
        }
        assert_eq!(result.equity.locked_margin[1_440], 0.0);
        assert_eq!(result.equity.option_liability[1_440], 0.0);
        assert_eq!(
            result.completed_trades.realized_pnl[0],
            result.completed_trades.total_received_premium[0]
                - result.completed_trades.total_expiry_payoff[0]
        );
        assert_eq!(result.summary.final_equity[0], result.equity.equity[1_440]);
        assert_eq!(
            result.summary.total_pnl[0],
            result.summary.final_equity[0] - initial
        );
    }
}

#[test]
fn flat_path_matches_the_hand_calculated_ledger() {
    let result = run_backtest(
        &timestamps(1_441),
        &flat(1_441, 100.0),
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();
    let received = result.executed_tranches.received_premium[0];

    assert_eq!(result.executed_tranches.quantity_steps[0], 70);
    assert_eq!(result.executed_tranches.quantity[0], 7.0);
    assert_eq!(result.equity.cash[0], 1_000.0 + received);
    assert_eq!(result.equity.option_liability[0], received);
    assert_eq!(result.equity.locked_margin[0], 700.0);
    assert_eq!(result.equity.available_margin[0], 300.0);
    assert_eq!(result.equity.equity[0], 1_000.0);

    assert_eq!(result.completed_trades.total_expiry_payoff[0], 0.0);
    assert_eq!(result.completed_trades.realized_pnl[0], received);
    assert_eq!(result.equity.cash[1_440], 1_000.0 + received);
    assert_eq!(result.equity.option_liability[1_440], 0.0);
    assert_eq!(result.equity.locked_margin[1_440], 0.0);
    assert_eq!(result.equity.equity[1_440], 1_000.0 + received);
}

#[test]
fn running_peak_drawdown_and_negative_equity_follow_the_exact_formula() {
    let result = run_backtest(
        &timestamps(1_441),
        &linear(1_441, 100.0, 1_000.0),
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();
    let mut peak = config().initial_capital_usd;
    let mut maximum_usd = 0.0_f64;
    let mut maximum_pct = 0.0_f64;
    for index in 0..result.equity.equity.len() {
        let equity = result.equity.equity[index];
        peak = peak.max(equity);
        let drawdown = (peak - equity).max(0.0);
        let percentage = drawdown / peak;
        assert_eq!(result.equity.running_peak[index], peak);
        assert_eq!(result.equity.drawdown_usd[index], drawdown);
        assert_eq!(result.equity.drawdown_pct[index], percentage);
        maximum_usd = maximum_usd.max(drawdown);
        maximum_pct = maximum_pct.max(percentage);
    }
    assert!(result.summary.minimum_equity[0] < 0.0);
    assert!(result.summary.maximum_drawdown_pct[0] > 1.0);
    assert_eq!(result.summary.maximum_drawdown_usd[0], maximum_usd);
    assert_eq!(result.summary.maximum_drawdown_pct[0], maximum_pct);
    assert!(result.summary.any_margin_breach[0]);
}

#[test]
fn expiry_only_margin_breach_is_attributed_to_the_completed_trade() {
    let mut close = flat(1_441, 100.0);
    close[1_440] = 1_000.0;
    let result = run_backtest(
        &timestamps(1_441),
        &close,
        dataset(),
        config(),
        &[IvScenario::Baseline],
    )
    .unwrap();

    assert!(result.equity.margin_breached[1_440]);
    assert!(result.summary.any_margin_breach[0]);
    assert!(result.completed_trades.margin_breached_during_trade[0]);
}

#[test]
fn shock_is_causal_full_repricing_and_restarts_at_each_trade_boundary() {
    let after = 10;
    let result = run_backtest(
        &timestamps(2_881),
        &flat(2_881, 100.0),
        dataset(),
        config(),
        &[
            IvScenario::Stress3x {
                after_minutes: after,
            },
            IvScenario::Baseline,
            IvScenario::Stress2x {
                after_minutes: after,
            },
        ],
    )
    .unwrap();
    assert_eq!(
        result.summary.scenario_id,
        [
            ScenarioCode::Baseline,
            ScenarioCode::Stress2x,
            ScenarioCode::Stress3x
        ]
    );
    let block = 2_881;
    for (scenario_index, multiplier) in [(0, 1.0), (1, 2.0), (2, 3.0)] {
        let start = scenario_index * block;
        assert_eq!(result.equity.active_iv[start], config().base_iv);
        assert_eq!(
            result.equity.active_iv[start + usize::from(after) - 1],
            config().base_iv
        );
        assert_eq!(
            result.equity.active_iv[start + usize::from(after)],
            config().base_iv * multiplier
        );
        assert_eq!(result.equity.active_iv[start + 1_440], config().base_iv);
        assert_eq!(
            result.equity.active_iv[start + 1_440 + usize::from(after)],
            config().base_iv * multiplier
        );
    }
    let baseline_pre = result.equity.equity[after as usize - 1];
    assert_eq!(
        result.equity.equity[block + after as usize - 1],
        baseline_pre
    );
    assert_ne!(
        result.equity.option_liability[block + after as usize],
        result.equity.option_liability[after as usize]
    );
}

#[test]
fn reserve_attempts_cover_full_reduced_and_rejected_without_fake_tranches() {
    let mut full_config = config();
    full_config.base_iv = 0.2;
    let full = run_backtest(
        &timestamps(1_441),
        &flat(1_441, 100.0),
        dataset(),
        full_config,
        &[IvScenario::Stress2x {
            after_minutes: 1_439,
        }],
    )
    .unwrap();
    assert_eq!(full.reserve_attempts.outcome, [ReserveOutcome::Full]);
    assert_eq!(full.executed_tranches.tranche_kind.len(), 2);
    assert_eq!(
        full.executed_tranches.strike[0],
        full.executed_tranches.strike[1]
    );
    assert_eq!(
        full.executed_tranches.expiry_timestamp_ns[0],
        full.executed_tranches.expiry_timestamp_ns[1]
    );

    let mut reduced_config = config();
    reduced_config.base_iv = 0.2;
    let reduced = run_backtest(
        &timestamps(1_441),
        &flat(1_441, 100.0),
        dataset(),
        reduced_config,
        &[IvScenario::Stress2x { after_minutes: 1 }],
    )
    .unwrap();
    assert_eq!(reduced.reserve_attempts.outcome, [ReserveOutcome::Reduced]);
    assert!(reduced.reserve_attempts.executed_quantity_steps[0] > 0);
    assert!(
        reduced.reserve_attempts.executed_quantity_steps[0]
            < reduced.reserve_attempts.requested_quantity_steps[0]
    );

    let mut rejected_config = config();
    rejected_config.base_iv = 0.2;
    let rejected = run_backtest(
        &timestamps(1_441),
        &flat(1_441, 10_000.0),
        dataset(),
        rejected_config,
        &[IvScenario::Stress3x { after_minutes: 1 }],
    )
    .unwrap();
    assert_eq!(
        rejected.reserve_attempts.outcome,
        [ReserveOutcome::Rejected]
    );
    assert_eq!(rejected.executed_tranches.tranche_kind.len(), 1);
    assert_eq!(rejected.summary.rejected_reserve_count, [1]);
}

#[test]
fn identical_runs_are_bitwise_equal_including_all_result_ordering() {
    let arguments = (
        timestamps(2_881),
        linear(2_881, 100.0, 105.0),
        dataset(),
        config(),
        vec![
            IvScenario::Stress2x { after_minutes: 720 },
            IvScenario::Baseline,
        ],
    );
    let first = run_backtest(
        &arguments.0,
        &arguments.1,
        arguments.2.clone(),
        arguments.3,
        &arguments.4,
    )
    .unwrap();
    let second = run_backtest(
        &arguments.0,
        &arguments.1,
        arguments.2.clone(),
        arguments.3,
        &arguments.4,
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first.summary.terminal_status,
        [TerminalStatus::Completed, TerminalStatus::Completed]
    );
}

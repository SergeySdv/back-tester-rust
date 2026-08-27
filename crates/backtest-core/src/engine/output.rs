use crate::margin::money_tolerance;
use crate::result::{
    BacktestResult, EquitySeries, ReserveOutcome, ReserveReason, ScenarioCode, TrancheKind,
};
use crate::{BacktestConfig, DomainError, IvScenario};

use super::accounting::{classify_margin, scenario_iv};
use super::state::ScenarioState;

pub(super) fn append_equity_row(
    index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    scenario: IvScenario,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    classify_margin(state)?;
    let equity = state.equity();
    let available = state.available_margin();
    state.running_peak = state.running_peak.max(equity);
    let drawdown_usd = (state.running_peak - equity).max(0.0);
    let drawdown_pct = drawdown_usd / state.running_peak;
    state.maximum_drawdown_usd = state.maximum_drawdown_usd.max(drawdown_usd);
    state.maximum_drawdown_pct = state.maximum_drawdown_pct.max(drawdown_pct);
    state.minimum_equity = state.minimum_equity.min(equity);
    state.minimum_available_margin = state.minimum_available_margin.min(available);
    state.maximum_locked_margin = state.maximum_locked_margin.max(state.locked_margin);

    let active = state.active_trade.as_ref();
    let active_iv = active.map(|trade| {
        scenario_iv(
            scenario,
            config.base_iv,
            index.saturating_sub(trade.entry_index),
        )
    });
    let table = &mut result.equity;
    table.timestamp_ns.push(timestamps_ns[index]);
    table.scenario_id.push(state.code);
    table.spot.push(close[index]);
    table
        .active_trade_id
        .push(active.map_or(0, |trade| trade.trade_id));
    table.active_trade_id_valid.push(active.is_some());
    table.active_iv.push(active_iv.unwrap_or(0.0));
    table.active_iv_valid.push(active_iv.is_some());
    table.cash.push(state.cash);
    table.option_liability.push(state.liability);
    table.locked_margin.push(state.locked_margin);
    table.available_margin.push(available);
    table.equity.push(equity);
    table.pnl.push(equity - config.initial_capital_usd);
    table.running_peak.push(state.running_peak);
    table.drawdown_usd.push(drawdown_usd);
    table.drawdown_pct.push(drawdown_pct);
    table
        .margin_breached
        .push(available < -money_tolerance(available, state.locked_margin));
    Ok(())
}

pub(super) fn append_summary(
    timestamps_ns: &[i64],
    config: BacktestConfig,
    state: &ScenarioState,
    result: &mut BacktestResult,
) {
    let final_equity = state.equity();
    let ignored = timestamps_ns.len() - state.processed_rows;
    let completed_count = result
        .completed_trades
        .scenario_id
        .iter()
        .filter(|code| **code == state.code)
        .count() as u64;
    let attempts = state.full_reserves + state.reduced_reserves + state.rejected_reserves;
    let table = &mut result.summary;
    table.scenario_id.push(state.code);
    table.terminal_status.push(state.terminal_status);
    table
        .terminal_timestamp_ns
        .push(timestamps_ns[state.processed_rows - 1]);
    table.initial_equity.push(config.initial_capital_usd);
    table.final_equity.push(final_equity);
    table
        .total_pnl
        .push(final_equity - config.initial_capital_usd);
    table
        .return_pct
        .push((final_equity - config.initial_capital_usd) / config.initial_capital_usd);
    table.maximum_drawdown_usd.push(state.maximum_drawdown_usd);
    table.maximum_drawdown_pct.push(state.maximum_drawdown_pct);
    table.minimum_equity.push(state.minimum_equity);
    table
        .minimum_available_margin
        .push(state.minimum_available_margin);
    table
        .maximum_locked_margin
        .push(state.maximum_locked_margin);
    table.completed_trade_count.push(completed_count);
    table.reserve_attempt_count.push(attempts);
    table.full_reserve_count.push(state.full_reserves);
    table.reduced_reserve_count.push(state.reduced_reserves);
    table.rejected_reserve_count.push(state.rejected_reserves);
    table
        .skipped_incomplete_window_count
        .push(state.skipped_tail);
    table.input_row_count.push(timestamps_ns.len() as u64);
    table.processed_row_count.push(state.processed_rows as u64);
    table.ignored_input_row_count.push(ignored as u64);
    table.any_margin_breach.push(state.any_margin_breach);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_tranche(
    result: &mut BacktestResult,
    scenario: ScenarioCode,
    trade_id: u64,
    tranche_id: u64,
    kind: TrancheKind,
    execution_timestamp_ns: i64,
    expiry_timestamp_ns: i64,
    strike: f64,
    steps: u64,
    quantity: f64,
    active_iv: f64,
    call: f64,
    put: f64,
    received: f64,
    locked: f64,
) {
    let table = &mut result.executed_tranches;
    table.scenario_id.push(scenario);
    table.trade_id.push(trade_id);
    table.tranche_id.push(tranche_id);
    table.tranche_kind.push(kind);
    table.execution_timestamp_ns.push(execution_timestamp_ns);
    table.expiry_timestamp_ns.push(expiry_timestamp_ns);
    table.strike.push(strike);
    table.quantity_steps.push(steps);
    table.quantity.push(quantity);
    table.active_iv.push(active_iv);
    table.call_premium_per_unit.push(call);
    table.put_premium_per_unit.push(put);
    table.total_premium_per_unit.push(call + put);
    table.received_premium.push(received);
    table.locked_margin.push(locked);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_reserve_attempt(
    result: &mut BacktestResult,
    scenario: ScenarioCode,
    trade_id: u64,
    timestamp_ns: i64,
    requested_steps: u64,
    executed_steps: u64,
    requested_quantity: f64,
    executed_quantity: f64,
    available_margin: f64,
    reserve_budget: f64,
    outcome: ReserveOutcome,
    reason: ReserveReason,
) {
    let table = &mut result.reserve_attempts;
    table.scenario_id.push(scenario);
    table.trade_id.push(trade_id);
    table.attempt_timestamp_ns.push(timestamp_ns);
    table.requested_quantity_steps.push(requested_steps);
    table.executed_quantity_steps.push(executed_steps);
    table.requested_quantity.push(requested_quantity);
    table.executed_quantity.push(executed_quantity);
    table.available_margin_before.push(available_margin);
    table.reserve_budget_remaining.push(reserve_budget);
    table.outcome.push(outcome);
    table.reason.push(reason);
}

pub(super) fn reserve_equity_capacity(table: &mut EquitySeries, capacity: usize) {
    table.timestamp_ns.reserve(capacity);
    table.scenario_id.reserve(capacity);
    table.spot.reserve(capacity);
    table.active_trade_id.reserve(capacity);
    table.active_trade_id_valid.reserve(capacity);
    table.active_iv.reserve(capacity);
    table.active_iv_valid.reserve(capacity);
    table.cash.reserve(capacity);
    table.option_liability.reserve(capacity);
    table.locked_margin.reserve(capacity);
    table.available_margin.reserve(capacity);
    table.equity.reserve(capacity);
    table.pnl.reserve(capacity);
    table.running_peak.reserve(capacity);
    table.drawdown_usd.reserve(capacity);
    table.drawdown_pct.reserve(capacity);
    table.margin_breached.reserve(capacity);
}

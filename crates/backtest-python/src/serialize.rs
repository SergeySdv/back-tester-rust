use backtest_core::{
    BacktestResult, CompletedTrades, EquitySeries, ExecutedTranches, ReserveAttempts,
    ScenarioSummary,
};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

macro_rules! dictionary {
    ($py:expr, $( $name:literal => $value:expr ),+ $(,)?) => {{
        let output = PyDict::new($py);
        $(output.set_item($name, $value)?;)+
        Ok(output)
    }};
}

fn strings<'py, T: Copy>(
    py: Python<'py>,
    values: &[T],
    render: impl Fn(T) -> &'static str,
) -> PyResult<Bound<'py, PyList>> {
    PyList::new(py, values.iter().copied().map(render))
}

fn equity<'py>(py: Python<'py>, v: &EquitySeries) -> PyResult<Bound<'py, PyDict>> {
    dictionary!(py,
        "timestamp_ns" => &v.timestamp_ns, "scenario_id" => strings(py, &v.scenario_id, |x| x.as_str())?,
        "spot" => &v.spot, "active_trade_id" => &v.active_trade_id,
        "active_trade_id_valid" => &v.active_trade_id_valid, "active_iv" => &v.active_iv,
        "active_iv_valid" => &v.active_iv_valid, "cash" => &v.cash,
        "option_liability" => &v.option_liability, "locked_margin" => &v.locked_margin,
        "available_margin" => &v.available_margin, "equity" => &v.equity, "pnl" => &v.pnl,
        "running_peak" => &v.running_peak, "drawdown_usd" => &v.drawdown_usd,
        "drawdown_pct" => &v.drawdown_pct, "margin_breached" => &v.margin_breached,
    )
}

fn trades<'py>(py: Python<'py>, v: &CompletedTrades) -> PyResult<Bound<'py, PyDict>> {
    dictionary!(py,
        "scenario_id" => strings(py, &v.scenario_id, |x| x.as_str())?, "trade_id" => &v.trade_id,
        "entry_timestamp_ns" => &v.entry_timestamp_ns, "expiry_timestamp_ns" => &v.expiry_timestamp_ns,
        "strike" => &v.strike, "entry_equity" => &v.entry_equity,
        "initial_quantity_steps" => &v.initial_quantity_steps, "initial_quantity" => &v.initial_quantity,
        "reserve_attempted" => &v.reserve_attempted,
        "reserve_executed_quantity_steps" => &v.reserve_executed_quantity_steps,
        "reserve_executed_quantity" => &v.reserve_executed_quantity,
        "total_received_premium" => &v.total_received_premium, "settlement_spot" => &v.settlement_spot,
        "total_expiry_payoff" => &v.total_expiry_payoff, "realized_pnl" => &v.realized_pnl,
        "margin_breached_during_trade" => &v.margin_breached_during_trade,
    )
}

fn tranches<'py>(py: Python<'py>, v: &ExecutedTranches) -> PyResult<Bound<'py, PyDict>> {
    dictionary!(py,
        "scenario_id" => strings(py, &v.scenario_id, |x| x.as_str())?, "trade_id" => &v.trade_id,
        "tranche_id" => &v.tranche_id, "tranche_kind" => strings(py, &v.tranche_kind, |x| x.as_str())?,
        "execution_timestamp_ns" => &v.execution_timestamp_ns,
        "expiry_timestamp_ns" => &v.expiry_timestamp_ns, "strike" => &v.strike,
        "quantity_steps" => &v.quantity_steps, "quantity" => &v.quantity, "active_iv" => &v.active_iv,
        "call_premium_per_unit" => &v.call_premium_per_unit,
        "put_premium_per_unit" => &v.put_premium_per_unit,
        "total_premium_per_unit" => &v.total_premium_per_unit,
        "received_premium" => &v.received_premium, "locked_margin" => &v.locked_margin,
    )
}

fn attempts<'py>(py: Python<'py>, v: &ReserveAttempts) -> PyResult<Bound<'py, PyDict>> {
    dictionary!(py,
        "scenario_id" => strings(py, &v.scenario_id, |x| x.as_str())?, "trade_id" => &v.trade_id,
        "attempt_timestamp_ns" => &v.attempt_timestamp_ns,
        "requested_quantity_steps" => &v.requested_quantity_steps,
        "executed_quantity_steps" => &v.executed_quantity_steps,
        "requested_quantity" => &v.requested_quantity, "executed_quantity" => &v.executed_quantity,
        "available_margin_before" => &v.available_margin_before,
        "reserve_budget_remaining" => &v.reserve_budget_remaining,
        "outcome" => strings(py, &v.outcome, |x| x.as_str())?,
        "reason" => strings(py, &v.reason, |x| x.as_str())?,
    )
}

fn summary<'py>(py: Python<'py>, v: &ScenarioSummary) -> PyResult<Bound<'py, PyDict>> {
    dictionary!(py,
        "scenario_id" => strings(py, &v.scenario_id, |x| x.as_str())?,
        "terminal_status" => strings(py, &v.terminal_status, |x| x.as_str())?,
        "terminal_timestamp_ns" => &v.terminal_timestamp_ns, "initial_equity" => &v.initial_equity,
        "final_equity" => &v.final_equity, "total_pnl" => &v.total_pnl, "return_pct" => &v.return_pct,
        "maximum_drawdown_usd" => &v.maximum_drawdown_usd,
        "maximum_drawdown_pct" => &v.maximum_drawdown_pct, "minimum_equity" => &v.minimum_equity,
        "minimum_available_margin" => &v.minimum_available_margin,
        "maximum_locked_margin" => &v.maximum_locked_margin,
        "completed_trade_count" => &v.completed_trade_count,
        "reserve_attempt_count" => &v.reserve_attempt_count, "full_reserve_count" => &v.full_reserve_count,
        "reduced_reserve_count" => &v.reduced_reserve_count,
        "rejected_reserve_count" => &v.rejected_reserve_count,
        "skipped_incomplete_window_count" => PyList::new(py, &v.skipped_incomplete_window_count)?,
        "input_row_count" => &v.input_row_count, "processed_row_count" => &v.processed_row_count,
        "ignored_input_row_count" => &v.ignored_input_row_count,
        "any_margin_breach" => &v.any_margin_breach,
    )
}

fn metadata<'py>(py: Python<'py>, value: &BacktestResult) -> PyResult<Bound<'py, PyDict>> {
    let m = &value.metadata;
    let scenarios = m
        .scenarios
        .iter()
        .map(|s| (s.scenario_id.as_str(), s.multiplier, s.shock_after_minutes))
        .collect::<Vec<_>>();
    dictionary!(py,
        "dataset_id" => &m.dataset.dataset_id, "source" => &m.dataset.source,
        "symbol" => &m.dataset.symbol, "interval_seconds" => m.dataset.interval_seconds,
        "timezone" => &m.dataset.timezone, "initial_capital_usd" => m.config.initial_capital_usd,
        "base_iv" => m.config.base_iv, "risk_free_rate" => m.config.risk_free_rate,
        "carry_rate" => m.config.carry_rate,
        "margin_per_straddle_usd" => m.config.margin_per_straddle_usd,
        "quantity_step" => m.config.quantity_step, "initial_allocation" => m.initial_allocation,
        "reserve_allocation" => m.reserve_allocation,
        "reserve_trigger_multiple" => m.reserve_trigger_multiple, "pricing_model" => m.pricing_model,
        "seconds_per_year" => m.seconds_per_year, "software_version" => m.software_version,
        "software_commit" => m.software_commit, "scenarios" => scenarios,
    )
}

pub(super) fn result<'py>(py: Python<'py>, value: &BacktestResult) -> PyResult<Bound<'py, PyDict>> {
    dictionary!(py,
        "metadata" => metadata(py, value)?, "equity" => equity(py, &value.equity)?,
        "trades" => trades(py, &value.completed_trades)?,
        "tranches" => tranches(py, &value.executed_tranches)?,
        "reserve_attempts" => attempts(py, &value.reserve_attempts)?,
        "summary" => summary(py, &value.summary)?,
    )
}

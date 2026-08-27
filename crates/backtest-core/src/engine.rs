use crate::margin::{
    INITIAL_ALLOCATION, RESERVE_ALLOCATION, RESERVE_TRIGGER_MULTIPLE, floor_steps, money_tolerance,
    reserve_triggered,
};
use crate::result::{
    BacktestResult, CompletedTrades, EquitySeries, ExecutedTranches, ReserveAttempts,
    ReserveOutcome, ReserveReason, RunMetadata, ScenarioCode, ScenarioMetadata, ScenarioSummary,
    TerminalStatus, TrancheKind,
};
use crate::{
    BacktestConfig, DatasetMetadata, DomainError, InvalidValue, IvScenario, SECONDS_PER_YEAR,
    black_scholes, expiry_payoff, validate_scenarios,
};

const WINDOW_MINUTES: usize = 24 * 60;
const MINUTE_NS: i64 = 60_000_000_000;
const PRICING_MODEL: &str = "synthetic Black-Scholes scenario backtest";

#[derive(Debug, Clone)]
struct ActiveTranche {
    quantity_steps: u64,
}

#[derive(Debug, Clone)]
struct ActiveTrade {
    trade_id: u64,
    entry_index: usize,
    expiry_index: usize,
    strike: f64,
    entry_equity: f64,
    initial_quantity_steps: u64,
    reserve_attempted: bool,
    reserve_executed_steps: u64,
    total_received_premium: f64,
    margin_breached: bool,
    tranches: Vec<ActiveTranche>,
}

struct ScenarioState {
    code: ScenarioCode,
    cash: f64,
    liability: f64,
    locked_margin: f64,
    running_peak: f64,
    maximum_drawdown_usd: f64,
    maximum_drawdown_pct: f64,
    minimum_equity: f64,
    minimum_available_margin: f64,
    maximum_locked_margin: f64,
    any_margin_breach: bool,
    next_trade_id: u64,
    next_tranche_id: u64,
    active_trade: Option<ActiveTrade>,
    terminal_status: TerminalStatus,
    processed_rows: usize,
    skipped_tail: u8,
    full_reserves: u64,
    reduced_reserves: u64,
    rejected_reserves: u64,
}

impl ScenarioState {
    fn new(code: ScenarioCode, initial_capital: f64) -> Self {
        Self {
            code,
            cash: initial_capital,
            liability: 0.0,
            locked_margin: 0.0,
            running_peak: initial_capital,
            maximum_drawdown_usd: 0.0,
            maximum_drawdown_pct: 0.0,
            minimum_equity: initial_capital,
            minimum_available_margin: initial_capital,
            maximum_locked_margin: 0.0,
            any_margin_breach: false,
            next_trade_id: 0,
            next_tranche_id: 0,
            active_trade: None,
            terminal_status: TerminalStatus::Completed,
            processed_rows: 0,
            skipped_tail: 0,
            full_reserves: 0,
            reduced_reserves: 0,
            rejected_reserves: 0,
        }
    }

    fn equity(&self) -> f64 {
        self.cash - self.liability
    }

    fn available_margin(&self) -> f64 {
        self.equity() - self.locked_margin
    }
}

/// Run all requested scenarios entirely inside the deterministic Rust core.
pub fn run_backtest(
    timestamps_ns: &[i64],
    close: &[f64],
    dataset: DatasetMetadata,
    config: BacktestConfig,
    scenarios: &[IvScenario],
) -> Result<BacktestResult, DomainError> {
    validate_inputs(timestamps_ns, close, &dataset, &config)?;
    let scenarios = validate_scenarios(scenarios)?;
    let metadata = RunMetadata {
        dataset,
        config,
        initial_allocation: INITIAL_ALLOCATION,
        reserve_allocation: RESERVE_ALLOCATION,
        reserve_trigger_multiple: RESERVE_TRIGGER_MULTIPLE,
        scenarios: scenarios
            .iter()
            .map(|scenario| ScenarioMetadata {
                scenario_id: (*scenario).into(),
                multiplier: scenario.multiplier(),
                shock_after_minutes: scenario.shock_after_minutes(),
            })
            .collect(),
        pricing_model: PRICING_MODEL,
        seconds_per_year: SECONDS_PER_YEAR,
        software_version: env!("CARGO_PKG_VERSION"),
        software_commit: option_env!("BACKTEST_GIT_COMMIT"),
    };
    let capacity = timestamps_ns.len().saturating_mul(scenarios.len());
    let mut result = BacktestResult {
        metadata,
        equity: EquitySeries::default(),
        completed_trades: CompletedTrades::default(),
        executed_tranches: ExecutedTranches::default(),
        reserve_attempts: ReserveAttempts::default(),
        summary: ScenarioSummary::default(),
    };
    reserve_equity_capacity(&mut result.equity, capacity);

    for scenario in scenarios {
        run_scenario(timestamps_ns, close, config, scenario, &mut result)?;
    }
    Ok(result)
}

fn validate_inputs(
    timestamps_ns: &[i64],
    close: &[f64],
    dataset: &DatasetMetadata,
    config: &BacktestConfig,
) -> Result<(), DomainError> {
    dataset.validate()?;
    config.validate()?;
    if timestamps_ns.len() != close.len() {
        return Err(DomainError::LengthMismatch {
            field: "close",
            expected: timestamps_ns.len(),
            actual: close.len(),
        });
    }
    if timestamps_ns.len() < WINDOW_MINUTES + 1 {
        return Err(DomainError::InsufficientHistory {
            minimum: WINDOW_MINUTES + 1,
            actual: timestamps_ns.len(),
        });
    }
    for index in 1..timestamps_ns.len() {
        if timestamps_ns[index].checked_sub(timestamps_ns[index - 1]) != Some(MINUTE_NS) {
            return Err(DomainError::InvalidTimestamp {
                index,
                previous_ns: timestamps_ns[index - 1],
                current_ns: timestamps_ns[index],
            });
        }
    }
    for (index, price) in close.iter().copied().enumerate() {
        let reason = if !price.is_finite() {
            Some(InvalidValue::NotFinite)
        } else if price <= 0.0 {
            Some(InvalidValue::MustBePositive)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(DomainError::InvalidPrice { index, reason });
        }
    }
    Ok(())
}

fn run_scenario(
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    scenario: IvScenario,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    let mut state = ScenarioState::new(scenario.into(), config.initial_capital_usd);
    open_trade(0, timestamps_ns, close, config, &mut state, result, true)?;
    let mut index = 0;
    loop {
        let expiring = state
            .active_trade
            .as_ref()
            .is_some_and(|trade| trade.expiry_index == index);
        if expiring {
            settle_trade(index, timestamps_ns, close, config, &mut state, result)?;
            if index + WINDOW_MINUTES < timestamps_ns.len() {
                if !open_trade(
                    index,
                    timestamps_ns,
                    close,
                    config,
                    &mut state,
                    result,
                    false,
                )? {
                    state.terminal_status = TerminalStatus::CapitalExhausted;
                }
            } else if index + 1 < timestamps_ns.len() {
                state.skipped_tail = 1;
            }
        } else if state.active_trade.is_some() && index > 0 {
            mark_and_maybe_reserve(
                index,
                timestamps_ns,
                close,
                config,
                scenario,
                &mut state,
                result,
            )?;
        }

        append_equity_row(
            index,
            timestamps_ns,
            close,
            config,
            scenario,
            &mut state,
            result,
        )?;
        state.processed_rows += 1;

        if state.active_trade.is_none() {
            break;
        }
        index += 1;
    }
    append_summary(timestamps_ns, config, &state, result);
    Ok(())
}

fn open_trade(
    entry_index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
    first: bool,
) -> Result<bool, DomainError> {
    let entry_equity = state.equity();
    if entry_equity <= 0.0 && !first {
        return Ok(false);
    }
    let steps = floor_steps(
        entry_equity * INITIAL_ALLOCATION,
        config.margin_per_straddle_usd,
        config.quantity_step,
    )?;
    if steps == 0 {
        if first {
            return Err(DomainError::InsufficientInitialCapital);
        }
        return Ok(false);
    }
    let expiry_index = entry_index + WINDOW_MINUTES;
    let strike = close[entry_index];
    let quantity = checked_quantity(steps, config.quantity_step)?;
    let prices = black_scholes(
        strike,
        strike,
        time_years(timestamps_ns[entry_index], timestamps_ns[expiry_index])?,
        config.base_iv,
        config.risk_free_rate,
        config.carry_rate,
    )?;
    let received = checked_product(prices.call + prices.put, quantity, "received_premium")?;
    let locked = checked_product(quantity, config.margin_per_straddle_usd, "locked_margin")?;
    let trade_id = state.next_trade_id;
    state.next_trade_id += 1;
    state.cash += received;
    state.liability += received;
    state.locked_margin += locked;
    let tranche_id = state.next_tranche_id;
    state.next_tranche_id += 1;
    append_tranche(
        result,
        state.code,
        trade_id,
        tranche_id,
        TrancheKind::Initial,
        timestamps_ns[entry_index],
        timestamps_ns[expiry_index],
        strike,
        steps,
        quantity,
        config.base_iv,
        prices.call,
        prices.put,
        received,
        locked,
    );
    state.active_trade = Some(ActiveTrade {
        trade_id,
        entry_index,
        expiry_index,
        strike,
        entry_equity,
        initial_quantity_steps: steps,
        reserve_attempted: false,
        reserve_executed_steps: 0,
        total_received_premium: received,
        margin_breached: false,
        tranches: vec![ActiveTranche {
            quantity_steps: steps,
        }],
    });
    Ok(true)
}

fn mark_and_maybe_reserve(
    index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    scenario: IvScenario,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    let trade = state
        .active_trade
        .as_ref()
        .ok_or(DomainError::NumericOverflow {
            field: "active_trade",
        })?;
    let elapsed_minutes = index - trade.entry_index;
    let active_iv = scenario_iv(scenario, config.base_iv, elapsed_minutes);
    let remaining = time_years(timestamps_ns[index], timestamps_ns[trade.expiry_index])?;
    let prices = black_scholes(
        close[index],
        trade.strike,
        remaining,
        active_iv,
        config.risk_free_rate,
        config.carry_rate,
    )?;
    let total_steps = trade
        .tranches
        .iter()
        .try_fold(0_u64, |total, tranche| {
            total.checked_add(tranche.quantity_steps)
        })
        .ok_or(DomainError::NumericOverflow {
            field: "total_quantity_steps",
        })?;
    let total_quantity = checked_quantity(total_steps, config.quantity_step)?;
    state.liability =
        checked_product(prices.call + prices.put, total_quantity, "option_liability")?;

    let should_attempt = !trade.reserve_attempted && reserve_triggered(active_iv, config.base_iv);
    if should_attempt {
        attempt_reserve(
            index,
            timestamps_ns,
            config,
            active_iv,
            prices.call,
            prices.put,
            state,
            result,
        )?;
    }
    classify_margin(state)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn attempt_reserve(
    index: usize,
    timestamps_ns: &[i64],
    config: BacktestConfig,
    active_iv: f64,
    call: f64,
    put: f64,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    let available = state.available_margin();
    let trade = state
        .active_trade
        .as_mut()
        .ok_or(DomainError::NumericOverflow {
            field: "active_trade",
        })?;
    trade.reserve_attempted = true;
    let reserve_budget = trade.entry_equity * RESERVE_ALLOCATION;
    let requested_steps = floor_steps(
        reserve_budget,
        config.margin_per_straddle_usd,
        config.quantity_step,
    )?;
    let executable_budget = reserve_budget.min(available).max(0.0);
    let executed_steps = floor_steps(
        executable_budget,
        config.margin_per_straddle_usd,
        config.quantity_step,
    )?;
    let (outcome, reason) = if executed_steps == requested_steps && executed_steps > 0 {
        state.full_reserves += 1;
        (ReserveOutcome::Full, ReserveReason::None)
    } else if executed_steps > 0 {
        state.reduced_reserves += 1;
        (
            ReserveOutcome::Reduced,
            ReserveReason::LimitedByAvailableMargin,
        )
    } else {
        state.rejected_reserves += 1;
        let reason = if available <= money_tolerance(available, 0.0) {
            ReserveReason::NoAvailableMargin
        } else {
            ReserveReason::BelowQuantityStep
        };
        (ReserveOutcome::Rejected, reason)
    };
    let requested_quantity = checked_quantity(requested_steps, config.quantity_step)?;
    let executed_quantity = checked_quantity(executed_steps, config.quantity_step)?;
    append_reserve_attempt(
        result,
        state.code,
        trade.trade_id,
        timestamps_ns[index],
        requested_steps,
        executed_steps,
        requested_quantity,
        executed_quantity,
        available,
        reserve_budget,
        outcome,
        reason,
    );
    if executed_steps > 0 {
        let received = checked_product(call + put, executed_quantity, "received_premium")?;
        let locked = checked_product(
            executed_quantity,
            config.margin_per_straddle_usd,
            "locked_margin",
        )?;
        state.cash += received;
        state.liability += received;
        state.locked_margin += locked;
        trade.reserve_executed_steps = executed_steps;
        trade.total_received_premium += received;
        trade.tranches.push(ActiveTranche {
            quantity_steps: executed_steps,
        });
        let tranche_id = state.next_tranche_id;
        state.next_tranche_id += 1;
        append_tranche(
            result,
            state.code,
            trade.trade_id,
            tranche_id,
            TrancheKind::Reserve,
            timestamps_ns[index],
            timestamps_ns[trade.expiry_index],
            trade.strike,
            executed_steps,
            executed_quantity,
            active_iv,
            call,
            put,
            received,
            locked,
        );
    }
    Ok(())
}

fn settle_trade(
    index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    let mut trade = state
        .active_trade
        .take()
        .ok_or(DomainError::NumericOverflow {
            field: "active_trade",
        })?;
    let total_steps = trade
        .tranches
        .iter()
        .try_fold(0_u64, |total, tranche| {
            total.checked_add(tranche.quantity_steps)
        })
        .ok_or(DomainError::NumericOverflow {
            field: "total_quantity_steps",
        })?;
    let quantity = checked_quantity(total_steps, config.quantity_step)?;
    let payoff = expiry_payoff(close[index], trade.strike)?;
    let total_payoff = checked_product(payoff.call + payoff.put, quantity, "expiry_payoff")?;
    state.cash -= total_payoff;
    state.liability = 0.0;
    state.locked_margin = 0.0;
    if margin_is_breached(state)? {
        state.any_margin_breach = true;
        trade.margin_breached = true;
    }
    let realized = trade.total_received_premium - total_payoff;
    let initial_quantity = checked_quantity(trade.initial_quantity_steps, config.quantity_step)?;
    let reserve_quantity = checked_quantity(trade.reserve_executed_steps, config.quantity_step)?;
    let table = &mut result.completed_trades;
    table.scenario_id.push(state.code);
    table.trade_id.push(trade.trade_id);
    table
        .entry_timestamp_ns
        .push(timestamps_ns[trade.entry_index]);
    table.expiry_timestamp_ns.push(timestamps_ns[index]);
    table.strike.push(trade.strike);
    table.entry_equity.push(trade.entry_equity);
    table
        .initial_quantity_steps
        .push(trade.initial_quantity_steps);
    table.initial_quantity.push(initial_quantity);
    table.reserve_attempted.push(trade.reserve_attempted);
    table
        .reserve_executed_quantity_steps
        .push(trade.reserve_executed_steps);
    table.reserve_executed_quantity.push(reserve_quantity);
    table
        .total_received_premium
        .push(trade.total_received_premium);
    table.settlement_spot.push(close[index]);
    table.total_expiry_payoff.push(total_payoff);
    table.realized_pnl.push(realized);
    table
        .margin_breached_during_trade
        .push(trade.margin_breached);
    Ok(())
}

fn append_equity_row(
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

fn append_summary(
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

fn classify_margin(state: &mut ScenarioState) -> Result<(), DomainError> {
    let breached = margin_is_breached(state)?;
    if breached {
        state.any_margin_breach = true;
        if let Some(trade) = state.active_trade.as_mut() {
            trade.margin_breached = true;
        }
    }
    Ok(())
}

fn margin_is_breached(state: &ScenarioState) -> Result<bool, DomainError> {
    let equity = state.equity();
    if !equity.is_finite() {
        return Err(DomainError::NumericOverflow { field: "equity" });
    }
    let available = state.available_margin();
    if !available.is_finite() {
        return Err(DomainError::NumericOverflow {
            field: "available_margin",
        });
    }
    Ok(available < -money_tolerance(available, state.locked_margin))
}

fn scenario_iv(scenario: IvScenario, base_iv: f64, elapsed_minutes: usize) -> f64 {
    match scenario.shock_after_minutes() {
        Some(after) if elapsed_minutes >= usize::from(after) => base_iv * scenario.multiplier(),
        _ => base_iv,
    }
}

fn time_years(timestamp_ns: i64, expiry_timestamp_ns: i64) -> Result<f64, DomainError> {
    let remaining_ns =
        expiry_timestamp_ns
            .checked_sub(timestamp_ns)
            .ok_or(DomainError::NumericOverflow {
                field: "time_to_expiry",
            })?;
    if remaining_ns < 0 {
        return Err(DomainError::NumericOverflow {
            field: "time_to_expiry",
        });
    }
    Ok(remaining_ns as f64 / 1_000_000_000.0 / SECONDS_PER_YEAR as f64)
}

fn checked_quantity(steps: u64, quantity_step: f64) -> Result<f64, DomainError> {
    checked_product(steps as f64, quantity_step, "quantity")
}

fn checked_product(a: f64, b: f64, field: &'static str) -> Result<f64, DomainError> {
    let value = a * b;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(DomainError::NumericOverflow { field })
    }
}

#[allow(clippy::too_many_arguments)]
fn append_tranche(
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
fn append_reserve_attempt(
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

fn reserve_equity_capacity(table: &mut EquitySeries, capacity: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut state = ScenarioState::new(ScenarioCode::Baseline, 1.0);
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
}

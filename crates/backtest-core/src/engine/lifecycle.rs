use crate::margin::{
    INITIAL_ALLOCATION, RESERVE_ALLOCATION, floor_steps, money_tolerance, reserve_triggered,
};
use crate::result::{BacktestResult, ReserveOutcome, ReserveReason, TerminalStatus, TrancheKind};
use crate::{BacktestConfig, DomainError, IvScenario, black_scholes, expiry_payoff};

use super::accounting::{
    checked_product, checked_quantity, classify_margin, margin_is_breached, scenario_iv,
    time_years, total_quantity_steps,
};

use super::WINDOW_MINUTES;
use super::output::{append_equity_row, append_reserve_attempt, append_summary, append_tranche};
use super::state::{ActiveTrade, ActiveTranche, ScenarioState};

pub(super) fn run_scenario(
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
        process_minute(
            index,
            timestamps_ns,
            close,
            config,
            scenario,
            &mut state,
            result,
        )?;
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

#[allow(clippy::too_many_arguments)]
fn process_minute(
    index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    scenario: IvScenario,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    let expiring = state
        .active_trade
        .as_ref()
        .is_some_and(|trade| trade.expiry_index == index);
    if expiring {
        process_expiry(index, timestamps_ns, close, config, state, result)
    } else if state.active_trade.is_some() && index > 0 {
        mark_and_maybe_reserve(index, timestamps_ns, close, config, scenario, state, result)
    } else {
        Ok(())
    }
}

fn process_expiry(
    index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    settle_trade(index, timestamps_ns, close, config, state, result)?;
    if index + WINDOW_MINUTES < timestamps_ns.len() {
        let opened = open_trade(index, timestamps_ns, close, config, state, result, false)?;
        if !opened {
            state.terminal_status = TerminalStatus::CapitalExhausted;
        }
    } else if index + 1 < timestamps_ns.len() {
        state.skipped_tail = 1;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    create_initial_tranche(
        entry_index,
        timestamps_ns,
        close,
        config,
        state,
        result,
        steps,
        entry_equity,
    )?;
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn create_initial_tranche(
    entry_index: usize,
    timestamps_ns: &[i64],
    close: &[f64],
    config: BacktestConfig,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
    steps: u64,
    entry_equity: f64,
) -> Result<(), DomainError> {
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
    let tranche_id = state.next_tranche_id;
    state.next_trade_id += 1;
    state.next_tranche_id += 1;
    state.cash += received;
    state.liability += received;
    state.locked_margin += locked;
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
    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
    let total_steps = total_quantity_steps(trade)?;
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
    classify_margin(state)
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
    let (trade_id, reserve_budget) = {
        let trade = state
            .active_trade
            .as_mut()
            .ok_or(DomainError::NumericOverflow {
                field: "active_trade",
            })?;
        trade.reserve_attempted = true;
        (trade.trade_id, trade.entry_equity * RESERVE_ALLOCATION)
    };
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
    let (outcome, reason) = classify_reserve(executed_steps, requested_steps, available, state);
    let requested_quantity = checked_quantity(requested_steps, config.quantity_step)?;
    let executed_quantity = checked_quantity(executed_steps, config.quantity_step)?;
    append_reserve_attempt(
        result,
        state.code,
        trade_id,
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
        execute_reserve(
            index,
            timestamps_ns,
            config,
            active_iv,
            call,
            put,
            executed_steps,
            executed_quantity,
            state,
            result,
        )?;
    }
    Ok(())
}

fn classify_reserve(
    executed_steps: u64,
    requested_steps: u64,
    available: f64,
    state: &mut ScenarioState,
) -> (ReserveOutcome, ReserveReason) {
    if executed_steps == requested_steps && executed_steps > 0 {
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
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_reserve(
    index: usize,
    timestamps_ns: &[i64],
    config: BacktestConfig,
    active_iv: f64,
    call: f64,
    put: f64,
    executed_steps: u64,
    executed_quantity: f64,
    state: &mut ScenarioState,
    result: &mut BacktestResult,
) -> Result<(), DomainError> {
    let received = checked_product(call + put, executed_quantity, "received_premium")?;
    let locked = checked_product(
        executed_quantity,
        config.margin_per_straddle_usd,
        "locked_margin",
    )?;
    let trade = state
        .active_trade
        .as_mut()
        .ok_or(DomainError::NumericOverflow {
            field: "active_trade",
        })?;
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
    let total_steps = total_quantity_steps(&trade)?;
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

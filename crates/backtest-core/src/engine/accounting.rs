use crate::margin::money_tolerance;
use crate::{DomainError, IvScenario, SECONDS_PER_YEAR};

use super::state::{ActiveTrade, ScenarioState};

pub(super) fn total_quantity_steps(trade: &ActiveTrade) -> Result<u64, DomainError> {
    trade
        .tranches
        .iter()
        .try_fold(0_u64, |total, tranche| {
            total.checked_add(tranche.quantity_steps)
        })
        .ok_or(DomainError::NumericOverflow {
            field: "total_quantity_steps",
        })
}

pub(super) fn classify_margin(state: &mut ScenarioState) -> Result<(), DomainError> {
    let breached = margin_is_breached(state)?;
    if breached {
        state.any_margin_breach = true;
        if let Some(trade) = state.active_trade.as_mut() {
            trade.margin_breached = true;
        }
    }
    Ok(())
}

pub(super) fn margin_is_breached(state: &ScenarioState) -> Result<bool, DomainError> {
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

pub(super) fn scenario_iv(scenario: IvScenario, base_iv: f64, elapsed_minutes: usize) -> f64 {
    match scenario.shock_after_minutes() {
        Some(after) if elapsed_minutes >= usize::from(after) => base_iv * scenario.multiplier(),
        _ => base_iv,
    }
}

pub(super) fn time_years(timestamp_ns: i64, expiry_timestamp_ns: i64) -> Result<f64, DomainError> {
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

pub(super) fn checked_quantity(steps: u64, quantity_step: f64) -> Result<f64, DomainError> {
    checked_product(steps as f64, quantity_step, "quantity")
}

pub(super) fn checked_product(a: f64, b: f64, field: &'static str) -> Result<f64, DomainError> {
    let value = a * b;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(DomainError::NumericOverflow { field })
    }
}

#[cfg(test)]
pub(super) fn state_with_code(
    code: crate::result::ScenarioCode,
    initial_capital: f64,
) -> ScenarioState {
    ScenarioState::new(code, initial_capital)
}

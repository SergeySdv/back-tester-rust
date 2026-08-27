use crate::DomainError;

pub const INITIAL_ALLOCATION: f64 = 0.70;
pub const RESERVE_ALLOCATION: f64 = 0.30;
pub const RESERVE_TRIGGER_MULTIPLE: f64 = 1.50;

pub fn money_tolerance(a: f64, b: f64) -> f64 {
    8.0 * f64::EPSILON * a.abs().max(b.abs()).max(1.0)
}

pub fn reserve_triggered(current_iv: f64, entry_iv: f64) -> bool {
    current_iv >= RESERVE_TRIGGER_MULTIPLE * entry_iv
}

pub fn floor_steps(
    budget: f64,
    margin_per_straddle_usd: f64,
    quantity_step: f64,
) -> Result<u64, DomainError> {
    if !budget.is_finite() || budget < 0.0 {
        return Err(DomainError::NumericOverflow { field: "budget" });
    }
    let margin_per_step = margin_per_straddle_usd * quantity_step;
    if !margin_per_step.is_finite() || margin_per_step <= 0.0 {
        return Err(DomainError::NumericOverflow {
            field: "margin_per_step",
        });
    }
    let raw_steps = budget / margin_per_step;
    if !raw_steps.is_finite() || raw_steps >= u64::MAX as f64 {
        return Err(DomainError::NumericOverflow {
            field: "quantity_steps",
        });
    }
    let nearest_integer = raw_steps.round();
    let ratio_tolerance = 8.0 * f64::EPSILON * raw_steps.abs().max(1.0);
    let normalized = if (raw_steps - nearest_integer).abs() <= ratio_tolerance {
        nearest_integer
    } else {
        raw_steps
    };
    let mut step_count = normalized.floor() as u64;
    loop {
        let step_margin = step_count as f64 * margin_per_step;
        if !step_margin.is_finite() {
            return Err(DomainError::NumericOverflow {
                field: "step_margin",
            });
        }
        if step_margin <= budget + money_tolerance(budget, step_margin) {
            return Ok(step_count);
        }
        step_count = step_count
            .checked_sub(1)
            .ok_or(DomainError::NumericOverflow {
                field: "quantity_steps",
            })?;
    }
}

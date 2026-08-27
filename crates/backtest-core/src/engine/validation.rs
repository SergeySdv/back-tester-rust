use crate::{BacktestConfig, DatasetMetadata, DomainError, InvalidValue};

use super::{MINUTE_NS, WINDOW_MINUTES};

pub(super) fn validate_inputs(
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
    validate_timestamps(timestamps_ns)?;
    validate_prices(close)
}

fn validate_timestamps(timestamps_ns: &[i64]) -> Result<(), DomainError> {
    for index in 1..timestamps_ns.len() {
        if timestamps_ns[index].checked_sub(timestamps_ns[index - 1]) != Some(MINUTE_NS) {
            return Err(DomainError::InvalidTimestamp {
                index,
                previous_ns: timestamps_ns[index - 1],
                current_ns: timestamps_ns[index],
            });
        }
    }
    Ok(())
}

fn validate_prices(close: &[f64]) -> Result<(), DomainError> {
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

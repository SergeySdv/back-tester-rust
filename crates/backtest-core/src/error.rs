use std::error::Error;
use std::fmt::{Display, Formatter};

/// The reason a numerical field failed validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidValue {
    NotFinite,
    MustBePositive,
    MustBeNonNegative,
    Empty,
    Unsupported,
    Duplicate,
}

/// Typed failures at the deterministic Rust domain boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InvalidField {
        field: &'static str,
        reason: InvalidValue,
    },
    InvalidScenario {
        index: usize,
        field: &'static str,
        reason: InvalidValue,
    },
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
}

impl Display for DomainError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidField { field, reason } => {
                write!(formatter, "invalid field `{field}`: {reason}")
            }
            Self::InvalidScenario {
                index,
                field,
                reason,
            } => write!(
                formatter,
                "invalid scenario at index {index}, field `{field}`: {reason}"
            ),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(
                formatter,
                "length mismatch for `{field}`: expected {expected}, got {actual}"
            ),
        }
    }
}

impl Display for InvalidValue {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NotFinite => "must be finite",
            Self::MustBePositive => "must be greater than zero",
            Self::MustBeNonNegative => "must be non-negative",
            Self::Empty => "must not be empty",
            Self::Unsupported => "has an unsupported value",
            Self::Duplicate => "must be unique",
        };
        formatter.write_str(message)
    }
}

impl Error for DomainError {}

pub(crate) fn require_finite(field: &'static str, value: f64) -> Result<(), DomainError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(DomainError::InvalidField {
            field,
            reason: InvalidValue::NotFinite,
        })
    }
}

pub(crate) fn require_positive(field: &'static str, value: f64) -> Result<(), DomainError> {
    require_finite(field, value)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(DomainError::InvalidField {
            field,
            reason: InvalidValue::MustBePositive,
        })
    }
}

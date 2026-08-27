use crate::error::{DomainError, InvalidValue, require_finite, require_positive};

/// Identity attached unchanged to future backtest results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetMetadata {
    pub dataset_id: String,
    pub source: String,
    pub symbol: String,
    pub interval_seconds: u32,
    pub timezone: String,
}

impl DatasetMetadata {
    pub fn validate(&self) -> Result<(), DomainError> {
        require_non_empty("dataset.dataset_id", &self.dataset_id)?;
        require_non_empty("dataset.source", &self.source)?;
        require_non_empty("dataset.symbol", &self.symbol)?;
        if self.interval_seconds != 60 {
            return Err(DomainError::InvalidField {
                field: "dataset.interval_seconds",
                reason: InvalidValue::Unsupported,
            });
        }
        if self.timezone != "UTC" {
            return Err(DomainError::InvalidField {
                field: "dataset.timezone",
                reason: InvalidValue::Unsupported,
            });
        }
        Ok(())
    }
}

/// Minimal configuration validated now and consumed by later engine epics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BacktestConfig {
    pub initial_capital_usd: f64,
    pub base_iv: f64,
    pub risk_free_rate: f64,
    pub carry_rate: f64,
    pub margin_per_straddle_usd: f64,
    pub quantity_step: f64,
}

impl BacktestConfig {
    pub fn validate(&self) -> Result<(), DomainError> {
        require_positive("config.initial_capital_usd", self.initial_capital_usd)?;
        require_positive("config.base_iv", self.base_iv)?;
        require_finite("config.risk_free_rate", self.risk_free_rate)?;
        require_finite("config.carry_rate", self.carry_rate)?;
        require_positive(
            "config.margin_per_straddle_usd",
            self.margin_per_straddle_usd,
        )?;
        require_positive("config.quantity_step", self.quantity_step)
    }
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        Err(DomainError::InvalidField {
            field,
            reason: InvalidValue::Empty,
        })
    } else {
        Ok(())
    }
}

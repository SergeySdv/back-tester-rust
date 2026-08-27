//! Deterministic domain primitives for the synthetic Black--Scholes backtester.

mod black_scholes;
mod config;
mod error;
mod scenario;

pub use black_scholes::{
    BatchOptionPrices, OptionPrices, PRICE_ABSOLUTE_TOLERANCE, PRICE_RELATIVE_TOLERANCE,
    SECONDS_PER_YEAR, black_scholes, black_scholes_many, expiry_payoff,
};
pub use config::{BacktestConfig, DatasetMetadata};
pub use error::{DomainError, InvalidValue};
pub use scenario::{IvScenario, validate_scenarios};

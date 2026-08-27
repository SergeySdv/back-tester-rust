//! Deterministic domain primitives for the synthetic Black--Scholes backtester.

mod black_scholes;
mod config;
mod engine;
mod error;
mod margin;
mod result;
mod scenario;

pub use black_scholes::{
    BatchOptionPrices, OptionPrices, PRICE_ABSOLUTE_TOLERANCE, PRICE_RELATIVE_TOLERANCE,
    SECONDS_PER_YEAR, black_scholes, black_scholes_many, expiry_payoff,
};
pub use config::{BacktestConfig, DatasetMetadata};
pub use engine::run_backtest;
pub use error::{DomainError, InvalidValue};
pub use margin::{
    INITIAL_ALLOCATION, RESERVE_ALLOCATION, RESERVE_TRIGGER_MULTIPLE, floor_steps, money_tolerance,
    reserve_triggered,
};
pub use result::{
    BacktestResult, COMPLETED_TRADES_SCHEMA, ColumnSchema, CompletedTrades, EQUITY_SCHEMA,
    EXECUTED_TRANCHES_SCHEMA, EquitySeries, ExecutedTranches, RESERVE_ATTEMPTS_SCHEMA,
    ReserveAttempts, ReserveOutcome, ReserveReason, RunMetadata, SUMMARY_SCHEMA, ScenarioCode,
    ScenarioMetadata, ScenarioSummary, TerminalStatus, TrancheKind,
};
pub use scenario::{IvScenario, validate_scenarios};

//! Deterministic orchestration for complete 24-hour scenarios.

mod accounting;
mod lifecycle;
mod output;
mod state;
mod validation;

use crate::margin::{INITIAL_ALLOCATION, RESERVE_ALLOCATION, RESERVE_TRIGGER_MULTIPLE};
use crate::result::{BacktestResult, EquitySeries, RunMetadata, ScenarioMetadata};
use crate::{
    BacktestConfig, DatasetMetadata, DomainError, IvScenario, SECONDS_PER_YEAR, validate_scenarios,
};
use lifecycle::run_scenario;
use output::reserve_equity_capacity;
use validation::validate_inputs;

pub(super) const WINDOW_MINUTES: usize = 24 * 60;
pub(super) const MINUTE_NS: i64 = 60_000_000_000;
const PRICING_MODEL: &str = "synthetic Black-Scholes scenario backtest";

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
        completed_trades: Default::default(),
        executed_tranches: Default::default(),
        reserve_attempts: Default::default(),
        summary: Default::default(),
    };
    reserve_equity_capacity(&mut result.equity, capacity);

    for scenario in scenarios {
        run_scenario(timestamps_ns, close, config, scenario, &mut result)?;
    }
    Ok(result)
}

#[cfg(test)]
mod tests;

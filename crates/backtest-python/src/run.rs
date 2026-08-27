use backtest_core::{BacktestConfig, DatasetMetadata, IvScenario, run_backtest};
use numpy::PyReadonlyArray1;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

use crate::{python_error, serialize};

fn missing(field: &str) -> PyErr {
    PyValueError::new_err(format!("missing native request field `{field}`"))
}

fn unsigned_range(field: &str, maximum: u64) -> PyErr {
    PyValueError::new_err(format!("{field} must be an integer in range 0..={maximum}"))
}

fn extract_u32(value: &Bound<'_, PyAny>, field: &str) -> PyResult<u32> {
    value
        .extract::<i128>()
        .ok()
        .and_then(|number| u32::try_from(number).ok())
        .ok_or_else(|| unsigned_range(field, u64::from(u32::MAX)))
}

fn extract_optional_u16(value: Option<Bound<'_, PyAny>>, field: &str) -> PyResult<Option<u16>> {
    value
        .map(|number| {
            number
                .extract::<i128>()
                .ok()
                .and_then(|number| u16::try_from(number).ok())
                .ok_or_else(|| unsigned_range(field, u64::from(u16::MAX)))
        })
        .transpose()
}

fn parse_dataset(values: &Bound<'_, PyDict>) -> PyResult<DatasetMetadata> {
    Ok(DatasetMetadata {
        dataset_id: values
            .get_item("dataset_id")?
            .ok_or_else(|| missing("dataset_id"))?
            .extract()?,
        source: values
            .get_item("source")?
            .ok_or_else(|| missing("source"))?
            .extract()?,
        symbol: values
            .get_item("symbol")?
            .ok_or_else(|| missing("symbol"))?
            .extract()?,
        interval_seconds: extract_u32(
            &values
                .get_item("interval_seconds")?
                .ok_or_else(|| missing("interval_seconds"))?,
            "interval_seconds",
        )?,
        timezone: values
            .get_item("timezone")?
            .ok_or_else(|| missing("timezone"))?
            .extract()?,
    })
}

fn parse_config(values: &Bound<'_, PyDict>) -> PyResult<BacktestConfig> {
    let get = |name| values.get_item(name)?.ok_or_else(|| missing(name));
    Ok(BacktestConfig {
        initial_capital_usd: get("initial_capital_usd")?.extract()?,
        base_iv: get("base_iv")?.extract()?,
        risk_free_rate: get("risk_free_rate")?.extract()?,
        carry_rate: get("carry_rate")?.extract()?,
        margin_per_straddle_usd: get("margin_per_straddle_usd")?.extract()?,
        quantity_step: get("quantity_step")?.extract()?,
    })
}

fn parse_scenarios(values: Vec<(String, Option<Bound<'_, PyAny>>)>) -> PyResult<Vec<IvScenario>> {
    values
        .into_iter()
        .map(|(scenario_id, shock)| {
            let shock = extract_optional_u16(shock, "shock_after_minutes")?;
            IvScenario::parse(&scenario_id, shock).map_err(python_error)
        })
        .collect()
}

/// Execute all requested scenarios in one call into the Rust state machine.
#[pyfunction(name = "run_backtest")]
pub(crate) fn run_backtest_native<'py>(
    py: Python<'py>,
    timestamps_ns: PyReadonlyArray1<'py, i64>,
    close: PyReadonlyArray1<'py, f64>,
    dataset: &Bound<'py, PyDict>,
    config: &Bound<'py, PyDict>,
    scenarios: Vec<(String, Option<Bound<'py, PyAny>>)>,
) -> PyResult<Bound<'py, PyDict>> {
    let timestamps_ns = timestamps_ns
        .as_slice()
        .map_err(|_| PyValueError::new_err("timestamps_ns must be C-contiguous"))?;
    let close = close
        .as_slice()
        .map_err(|_| PyValueError::new_err("close must be C-contiguous"))?;
    let result = run_backtest(
        timestamps_ns,
        close,
        parse_dataset(dataset)?,
        parse_config(config)?,
        &parse_scenarios(scenarios)?,
    )
    .map_err(python_error)?;
    serialize::result(py, &result)
}

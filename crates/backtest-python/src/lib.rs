use backtest_core::{DomainError, black_scholes_many};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

fn python_error(error: DomainError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Price a batch in one native call; all vectors must have equal length.
#[pyfunction]
fn price_many(
    spot: Vec<f64>,
    strike: Vec<f64>,
    time_years: Vec<f64>,
    sigma: Vec<f64>,
    risk_free_rate: f64,
    carry_rate: f64,
) -> PyResult<(Vec<f64>, Vec<f64>)> {
    let prices = black_scholes_many(
        &spot,
        &strike,
        &time_years,
        &sigma,
        risk_free_rate,
        carry_rate,
    )
    .map_err(python_error)?;
    Ok((prices.calls, prices.puts))
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(price_many, module)?)?;
    Ok(())
}

#![allow(dead_code)]

use backtest_core::{BacktestConfig, DatasetMetadata};

pub const MINUTE_NS: i64 = 60_000_000_000;

pub fn timestamps(points: usize) -> Vec<i64> {
    (0..points).map(|index| index as i64 * MINUTE_NS).collect()
}

pub fn flat(points: usize, price: f64) -> Vec<f64> {
    vec![price; points]
}

pub fn linear(points: usize, start: f64, end: f64) -> Vec<f64> {
    let denominator = (points - 1) as f64;
    (0..points)
        .map(|index| start + (end - start) * index as f64 / denominator)
        .collect()
}

pub fn dataset() -> DatasetMetadata {
    DatasetMetadata {
        dataset_id: "synthetic-fixture".into(),
        source: "test".into(),
        symbol: "BTCUSDT-SWAP".into(),
        interval_seconds: 60,
        timezone: "UTC".into(),
    }
}

pub fn config() -> BacktestConfig {
    BacktestConfig {
        initial_capital_usd: 1_000.0,
        base_iv: 0.55,
        risk_free_rate: 0.0,
        carry_rate: 0.0,
        margin_per_straddle_usd: 100.0,
        quantity_step: 0.1,
    }
}

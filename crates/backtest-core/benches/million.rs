use std::time::Instant;

use backtest_core::{BacktestConfig, DatasetMetadata, IvScenario, run_backtest};

const MINIMUM_PROCESSED_POINTS: u64 = 1_000_000;
const POINTS: usize = 1_000_801;
const MINUTE_NS: i64 = 60_000_000_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timestamps_ns: Vec<i64> = (0..POINTS).map(|index| index as i64 * MINUTE_NS).collect();
    let close = vec![100.0; POINTS];
    let started = Instant::now();
    let result = run_backtest(
        &timestamps_ns,
        &close,
        DatasetMetadata {
            dataset_id: "benchmark-million-flat".into(),
            source: "synthetic-benchmark".into(),
            symbol: "BTCUSDT-SWAP".into(),
            interval_seconds: 60,
            timezone: "UTC".into(),
        },
        BacktestConfig {
            initial_capital_usd: 1_000.0,
            base_iv: 0.55,
            risk_free_rate: 0.0,
            carry_rate: 0.0,
            margin_per_straddle_usd: 100.0,
            quantity_step: 0.1,
        },
        &[IvScenario::Baseline],
    )?;
    let elapsed = started.elapsed();
    let processed_points = result.summary.processed_row_count[0];
    if processed_points < MINIMUM_PROCESSED_POINTS {
        return Err(format!(
            "benchmark processed {processed_points} points; expected at least {MINIMUM_PROCESSED_POINTS}"
        )
        .into());
    }
    println!(
        "environment: os={} arch={} profile=release logical_cpus={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism()?.get()
    );
    println!(
        "result: input_points={} processed_points={} completed_trades={} elapsed_seconds={:.6} points_per_second={:.0}",
        POINTS,
        processed_points,
        result.summary.completed_trade_count[0],
        elapsed.as_secs_f64(),
        processed_points as f64 / elapsed.as_secs_f64()
    );
    Ok(())
}

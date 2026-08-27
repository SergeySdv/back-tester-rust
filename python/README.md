# Python API and data boundary

`back_tester.run(...)` accepts one-dimensional C-contiguous NumPy arrays with
exact dtypes `int64` (`timestamps_ns`) and `float64` (`close`). One native call
runs the requested fixed scenario set; Python only converts returned Rust
buffers and validity masks into the five contract pandas tables.

`load_minutes(...)` reads CSV or Parquet only with an explicit `ColumnMapping`
and `DatasetMetadata`. Supported source timestamp units are `s`, `ms`, `us`,
and `ns`; values are checked before conversion to Unix nanoseconds. The loader
does not infer venue, instrument, columns, units, or timezone from a filename,
and never sorts, fills, deduplicates, or otherwise repairs input.

Every run is labeled `synthetic Black–Scholes scenario backtest`. Its option
marks are synthetic and its USD margin model is simplified; outputs do not
represent historical option execution or exchange-margin fidelity.

The reviewed OKX REST candle mapping and reproducible integration evidence are
documented in [`docs/research/okx_1m_integration_evidence.md`](../docs/research/okx_1m_integration_evidence.md).
`load_okx_history_candles(...)` additionally enforces the exact source schema,
SHA256 identity and `confirm=1` for every candle.

The complete setup, dataset, run, reconciliation and strategy-validation
workflow is documented in
[`docs/guides/run_backtest_and_validate_strategy.md`](../docs/guides/run_backtest_and_validate_strategy.md).

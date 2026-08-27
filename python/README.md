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

The exact OKX mapping and real-data acceptance remain blocked until a
representative CSV or Parquet file and its source metadata are supplied.

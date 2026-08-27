from __future__ import annotations

from io import BytesIO
from pathlib import Path
from typing import Any

import back_tester.data as data_module
import numpy as np
import pandas as pd
import pytest
from back_tester import (
    OKX_HISTORY_CANDLES_MAPPING,
    ColumnMapping,
    DatasetMetadata,
    load_minutes,
    load_okx_history_candles,
)
from back_tester.data import validate_run_arrays


def mapped() -> ColumnMapping:
    return ColumnMapping("event_time", "settlement_proxy", "s")


def metadata() -> DatasetMetadata:
    return DatasetMetadata("explicit-synthetic-id", "fixture", "EXPLICIT-PROXY", 60, "UTC")


def frame() -> pd.DataFrame:
    return pd.DataFrame({
        "event_time": np.arange(1_441, dtype=np.int64) * 60,
        "settlement_proxy": np.linspace(100.5, 101.5, 1_441, dtype=np.float64),
    })


@pytest.mark.parametrize("suffix", [".csv", ".parquet"])
def test_explicit_loader_accepts_csv_and_parquet(tmp_path: Path, suffix: str) -> None:
    path = tmp_path / f"unverified-name{suffix}"
    source = frame()
    if suffix == ".csv":
        source.to_csv(path, index=False)
    else:
        source.to_parquet(path, index=False)
    result = load_minutes(path, mapping=mapped(), metadata=metadata())
    assert result.metadata == metadata()
    assert result.timestamps_ns.dtype == np.dtype("int64")
    assert result.close.dtype == np.dtype("float64")
    assert result.timestamps_ns.flags.c_contiguous
    assert result.close.flags.c_contiguous
    assert result.timestamps_ns[1] == 60_000_000_000


def write_csv(tmp_path: Path, value: pd.DataFrame) -> Path:
    path = tmp_path / "input.csv"
    value.to_csv(path, index=False)
    return path


def okx_frame() -> pd.DataFrame:
    timestamps = 1_787_529_600_000 + np.arange(1_441, dtype=np.int64) * 60_000
    close = np.linspace(77_712.5, 78_010.0, 1_441, dtype=np.float64)
    return pd.DataFrame({
        "timestamp_ms": timestamps,
        "open": close, "high": close + 1.0, "low": close - 1.0, "close": close,
        "volume_contracts": np.ones(1_441, dtype=np.float64),
        "volume_base": np.ones(1_441, dtype=np.float64),
        "volume_quote": np.ones(1_441, dtype=np.float64),
        "confirm": np.ones(1_441, dtype=np.int64),
    })


def file_sha256(path: Path) -> str:
    import hashlib

    return hashlib.sha256(path.read_bytes()).hexdigest()


def frame_bytes(value: pd.DataFrame, suffix: str) -> bytes:
    output = BytesIO()
    if suffix == ".csv":
        value.to_csv(output, index=False)
    else:
        value.to_parquet(output, index=False)
    return output.getvalue()


def test_okx_history_candles_enforces_mapping_completion_and_identity(tmp_path: Path) -> None:
    path = write_csv(tmp_path, okx_frame())
    checksum = file_sha256(path)

    result = load_okx_history_candles(path, expected_sha256=checksum)

    assert OKX_HISTORY_CANDLES_MAPPING == ColumnMapping("timestamp_ms", "close", "ms")
    assert result.timestamps_ns[0] == 1_787_529_600_000_000_000
    assert result.close[0] == 77_712.5
    assert result.metadata.dataset_id.endswith(f"sha256:{checksum}")
    assert result.metadata.source == "OKX public REST v5 GET /api/v5/market/history-candles"
    assert result.metadata.symbol == "BTC-USDT-SWAP"
    assert result.metadata.interval_seconds == 60
    assert result.metadata.timezone == "UTC"


@pytest.mark.parametrize("suffix", [".csv", ".parquet"])
def test_okx_history_candles_hashes_and_parses_one_snapshot(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, suffix: str
) -> None:
    path = tmp_path / f"input{suffix}"
    original = okx_frame()
    replacement = okx_frame()
    replacement["close"] += 10_000.0
    original_bytes = frame_bytes(original, suffix)
    replacement_bytes = frame_bytes(replacement, suffix)
    path.write_bytes(original_bytes)
    original_snapshot = data_module._read_snapshot

    def snapshot_then_replace(source_path: Path) -> bytes:
        snapshot = original_snapshot(source_path)
        source_path.write_bytes(replacement_bytes)
        return snapshot

    monkeypatch.setattr(data_module, "_read_snapshot", snapshot_then_replace)
    checksum = file_sha256(path)
    result = load_okx_history_candles(path, expected_sha256=checksum)

    assert result.close[0] == original["close"].iloc[0]
    assert result.close[0] != replacement["close"].iloc[0]
    assert result.metadata.dataset_id.endswith(f"sha256:{checksum}")
    assert path.read_bytes() == replacement_bytes


def test_okx_history_candles_rejects_checksum_schema_and_incomplete_rows(
    tmp_path: Path,
) -> None:
    path = write_csv(tmp_path, okx_frame())
    with pytest.raises(ValueError, match="SHA256 mismatch"):
        load_okx_history_candles(path, expected_sha256="0" * 64)

    invalid = okx_frame().drop(columns="volume_quote")
    path = write_csv(tmp_path, invalid)
    with pytest.raises(ValueError, match="columns must exactly equal"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))

    invalid = okx_frame()
    invalid.loc[100, "confirm"] = 0
    path = write_csv(tmp_path, invalid)
    with pytest.raises(ValueError, match="incomplete OKX candle at index 100"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))

    invalid = okx_frame().astype({"confirm": "float64"})
    path = write_csv(tmp_path, invalid)
    with pytest.raises(TypeError, match="confirm must have dtype int64"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))

    invalid = okx_frame()
    invalid.loc[100, "timestamp_ms"] += 60_000
    path = write_csv(tmp_path, invalid)
    with pytest.raises(ValueError, match="invalid timestamp at index 100"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))


def test_okx_history_candles_rejects_reordered_schema_without_repair(
    tmp_path: Path,
) -> None:
    invalid = okx_frame()
    columns = invalid.columns.to_list()
    columns[1], columns[2] = columns[2], columns[1]
    invalid = invalid.loc[:, columns]
    path = write_csv(tmp_path, invalid)
    before = path.read_bytes()

    with pytest.raises(ValueError, match="columns must exactly equal"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))

    assert path.read_bytes() == before


@pytest.mark.parametrize(
    "column",
    [
        "open",
        "high",
        "low",
        "close",
        "volume_contracts",
        "volume_base",
        "volume_quote",
    ],
)
def test_okx_history_candles_rejects_non_numeric_ohlcv_dtype(
    tmp_path: Path, column: str
) -> None:
    invalid = okx_frame()
    invalid[column] = "not-a-number"
    path = write_csv(tmp_path, invalid)
    before = path.read_bytes()

    with pytest.raises(TypeError, match=rf"{column} must have dtype float64"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))

    assert path.read_bytes() == before


@pytest.mark.parametrize("column", ["timestamp_ms", "confirm"])
def test_okx_history_candles_rejects_non_integer_identity_dtype(
    tmp_path: Path, column: str
) -> None:
    invalid = okx_frame().astype({column: "float64"})
    path = write_csv(tmp_path, invalid)
    before = path.read_bytes()

    with pytest.raises(TypeError, match=rf"{column} must have dtype int64"):
        load_okx_history_candles(path, expected_sha256=file_sha256(path))

    assert path.read_bytes() == before


def test_tardis_raw_trades_cannot_be_silently_used_as_minute_rows(tmp_path: Path) -> None:
    trades = pd.DataFrame({
        "local_timestamp": np.arange(1_441, dtype=np.int64) * 1_000_000,
        "price": np.linspace(16_500.0, 16_600.0, 1_441, dtype=np.float64),
    })
    original = trades.copy(deep=True)

    with pytest.raises(ValueError, match="expected an exact 60-second step"):
        load_minutes(
            write_csv(tmp_path, trades),
            mapping=ColumnMapping("local_timestamp", "price", "us"),
            metadata=DatasetMetadata("tardis-trades", "Tardis raw trades", "BTC-USDT-SWAP", 60, "UTC"),
        )

    pd.testing.assert_frame_equal(trades, original)


@pytest.mark.parametrize("problem", ["gap", "duplicate", "out_of_order"])
def test_loader_rejects_timestamp_sequence_without_repair(tmp_path: Path, problem: str) -> None:
    source = frame()
    if problem == "gap":
        source.loc[20, "event_time"] += 60
    elif problem == "duplicate":
        source.loc[20, "event_time"] = source.loc[19, "event_time"]
    else:
        source.loc[20, "event_time"] = source.loc[18, "event_time"]
    original = source["event_time"].copy()
    with pytest.raises(ValueError, match="invalid timestamp at index"):
        load_minutes(write_csv(tmp_path, source), mapping=mapped(), metadata=metadata())
    pd.testing.assert_series_equal(source["event_time"], original)


@pytest.mark.parametrize("value", [0.0, -1.0, float("nan"), float("inf")])
def test_loader_rejects_invalid_close(tmp_path: Path, value: float) -> None:
    source = frame()
    source.loc[3, "settlement_proxy"] = value
    with pytest.raises(ValueError, match="invalid close at index 3"):
        load_minutes(write_csv(tmp_path, source), mapping=mapped(), metadata=metadata())


def test_loader_rejects_missing_columns_and_invalid_dtypes(tmp_path: Path) -> None:
    source = frame().drop(columns="settlement_proxy")
    with pytest.raises(ValueError, match="missing mapped columns: settlement_proxy"):
        load_minutes(write_csv(tmp_path, source), mapping=mapped(), metadata=metadata())

    source = frame().astype({"event_time": "float64"})
    with pytest.raises(TypeError, match="event_time must have dtype int64"):
        load_minutes(write_csv(tmp_path, source), mapping=mapped(), metadata=metadata())

    source = frame().astype({"settlement_proxy": "int64"})
    with pytest.raises(TypeError, match="settlement_proxy must have dtype float64"):
        load_minutes(write_csv(tmp_path, source), mapping=mapped(), metadata=metadata())


@pytest.mark.parametrize(
    ("mapping", "message"),
    [
        (ColumnMapping("", "close", "ns"), "must not be empty"),
        (ColumnMapping("same", "same", "ns"), "different columns"),
        (ColumnMapping("time", "close", "minutes"), "timestamp_unit"),
    ],
)
def test_loader_rejects_invalid_mapping(
    tmp_path: Path, mapping: ColumnMapping, message: str
) -> None:
    with pytest.raises(ValueError, match=message):
        load_minutes(tmp_path / "unused.csv", mapping=mapping, metadata=metadata())


@pytest.mark.parametrize(
    "bad_metadata",
    [
        DatasetMetadata("", "fixture", "symbol", 60, "UTC"),
        DatasetMetadata("id", "", "symbol", 60, "UTC"),
        DatasetMetadata("id", "fixture", "", 60, "UTC"),
        DatasetMetadata("id", "fixture", "symbol", 300, "UTC"),
        DatasetMetadata("id", "fixture", "symbol", 60, "Europe/Moscow"),
    ],
)
def test_loader_rejects_invalid_metadata(tmp_path: Path, bad_metadata: DatasetMetadata) -> None:
    with pytest.raises(ValueError, match="metadata"):
        load_minutes(tmp_path / "unused.csv", mapping=mapped(), metadata=bad_metadata)


def test_loader_rejects_unknown_format_and_timestamp_overflow(tmp_path: Path) -> None:
    with pytest.raises(ValueError, match="must end in"):
        load_minutes(tmp_path / "input.json", mapping=mapped(), metadata=metadata())
    source = frame()
    source["event_time"] = np.iinfo(np.int64).max
    with pytest.raises(ValueError, match="overflows int64"):
        load_minutes(write_csv(tmp_path, source), mapping=mapped(), metadata=metadata())


@pytest.mark.parametrize(
    ("timestamps", "close", "error"),
    [
        ([0] * 1_441, np.ones(1_441, dtype=np.float64), "numpy.ndarray"),
        (np.zeros(1_441, dtype=np.float64), np.ones(1_441), "dtype int64"),
        (np.zeros((1_441, 1), dtype=np.int64), np.ones(1_441), "one-dimensional"),
        (np.zeros(1_440, dtype=np.int64), np.ones(1_441), "lengths differ"),
        (np.zeros(1_440, dtype=np.int64), np.ones(1_440), "at least 1441"),
    ],
)
def test_public_array_contract_rejects_invalid_inputs(
    timestamps: Any, close: Any, error: str
) -> None:
    with pytest.raises((TypeError, ValueError), match=error):
        validate_run_arrays(timestamps, close)


def test_public_array_contract_rejects_noncontiguous() -> None:
    timestamps = np.arange(2_882, dtype=np.int64)[::2]
    close = np.ones(1_441, dtype=np.float64)
    with pytest.raises(ValueError, match="timestamps_ns must be C-contiguous"):
        validate_run_arrays(timestamps, close)

from __future__ import annotations

from pathlib import Path
from typing import Any

import numpy as np
import pandas as pd
import pytest
from back_tester import ColumnMapping, DatasetMetadata, load_minutes
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

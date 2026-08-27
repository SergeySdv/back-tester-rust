"""Strict CSV/Parquet loading with explicit source-backed column mapping."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Union

import numpy as np
import pandas as pd
from numpy.typing import NDArray

from .config import DatasetMetadata

MINUTE_NS = 60_000_000_000
MIN_POINTS = 1_441
TIMESTAMP_FACTORS = {"s": 1_000_000_000, "ms": 1_000_000, "us": 1_000, "ns": 1}


@dataclass(frozen=True)
class ColumnMapping:
    """Mapping verified from a source manifest or representative file."""

    timestamp_column: str
    close_column: str
    timestamp_unit: str


@dataclass(frozen=True)
class MinuteData:
    timestamps_ns: NDArray[np.int64]
    close: NDArray[np.float64]
    metadata: DatasetMetadata


def validate_run_arrays(
    timestamps_ns: NDArray[np.int64], close: NDArray[np.float64]
) -> None:
    """Validate the zero-copy-compatible public array boundary."""
    _require_array(timestamps_ns, "timestamps_ns", np.dtype("int64"))
    _require_array(close, "close", np.dtype("float64"))
    if len(timestamps_ns) != len(close):
        raise ValueError(
            f"timestamps_ns and close lengths differ: {len(timestamps_ns)} != {len(close)}"
        )
    if len(timestamps_ns) < MIN_POINTS:
        raise ValueError(f"at least {MIN_POINTS} minute points required, got {len(timestamps_ns)}")


def validate_minute_values(
    timestamps_ns: NDArray[np.int64], close: NDArray[np.float64]
) -> None:
    """Reject invalid data without sorting, filling, or deduplicating it."""
    validate_run_arrays(timestamps_ns, close)
    bad_price = np.flatnonzero(~np.isfinite(close) | (close <= 0.0))
    if bad_price.size:
        raise ValueError(f"invalid close at index {int(bad_price[0])}: must be finite and positive")
    gaps = np.flatnonzero(np.diff(timestamps_ns) != MINUTE_NS)
    if gaps.size:
        index = int(gaps[0] + 1)
        raise ValueError(f"invalid timestamp at index {index}: expected an exact 60-second step")


def load_minutes(
    path: Union[str, Path], *, mapping: ColumnMapping, metadata: DatasetMetadata
) -> MinuteData:
    """Load mapped columns; venue schema and timestamp unit are always explicit."""
    _validate_mapping(mapping)
    _validate_metadata(metadata)
    frame = _read_frame(Path(path))
    missing = [
        column
        for column in (mapping.timestamp_column, mapping.close_column)
        if column not in frame.columns
    ]
    if missing:
        raise ValueError(f"missing mapped columns: {', '.join(missing)}")
    timestamp_series = frame[mapping.timestamp_column]
    close_series = frame[mapping.close_column]
    if timestamp_series.dtype != np.dtype("int64"):
        raise TypeError(f"{mapping.timestamp_column} must have dtype int64")
    if close_series.dtype != np.dtype("float64"):
        raise TypeError(f"{mapping.close_column} must have dtype float64")
    timestamps_ns = _to_nanoseconds(timestamp_series.to_numpy(copy=False), mapping.timestamp_unit)
    close = np.ascontiguousarray(close_series.to_numpy(copy=False), dtype=np.float64)
    validate_minute_values(timestamps_ns, close)
    return MinuteData(timestamps_ns=timestamps_ns, close=close, metadata=metadata)


def _require_array(value: object, name: str, dtype: np.dtype) -> None:
    if not isinstance(value, np.ndarray):
        raise TypeError(f"{name} must be a numpy.ndarray")
    if value.ndim != 1:
        raise TypeError(f"{name} must be one-dimensional")
    if value.dtype != dtype:
        raise TypeError(f"{name} must have dtype {dtype.name}")
    if not value.flags.c_contiguous:
        raise ValueError(f"{name} must be C-contiguous")


def _validate_mapping(mapping: ColumnMapping) -> None:
    if not mapping.timestamp_column or not mapping.close_column:
        raise ValueError("mapped column names must not be empty")
    if mapping.timestamp_column == mapping.close_column:
        raise ValueError("timestamp and close mappings must name different columns")
    if mapping.timestamp_unit not in TIMESTAMP_FACTORS:
        raise ValueError("timestamp_unit must be one of: s, ms, us, ns")


def _validate_metadata(metadata: DatasetMetadata) -> None:
    for field in ("dataset_id", "source", "symbol"):
        if not str(getattr(metadata, field)).strip():
            raise ValueError(f"metadata.{field} must not be empty")
    if metadata.interval_seconds != 60:
        raise ValueError("metadata.interval_seconds must equal 60")
    if metadata.timezone != "UTC":
        raise ValueError("metadata.timezone must equal UTC")


def _read_frame(path: Path) -> pd.DataFrame:
    if path.suffix.lower() == ".csv":
        return pd.read_csv(path)
    if path.suffix.lower() in {".parquet", ".pq"}:
        return pd.read_parquet(path)
    raise ValueError("minute data path must end in .csv, .parquet, or .pq")


def _to_nanoseconds(values: NDArray[np.int64], unit: str) -> NDArray[np.int64]:
    factor = TIMESTAMP_FACTORS[unit]
    if values.size and factor != 1:
        minimum = int(np.iinfo(np.int64).min)
        maximum = int(np.iinfo(np.int64).max)
        lower = -((-minimum) // factor)
        upper = maximum // factor
        if int(values.min()) < lower or int(values.max()) > upper:
            raise ValueError("timestamp conversion to nanoseconds overflows int64")
    return np.ascontiguousarray(values * factor, dtype=np.int64)

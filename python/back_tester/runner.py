"""Single-call orchestration into the native Rust engine."""

from __future__ import annotations

from typing import List

import numpy as np
from numpy.typing import NDArray

from . import _native
from .config import BacktestConfig, DatasetMetadata, IvScenario, native_scenarios
from .data import validate_run_arrays
from .reporting import BacktestResult, from_native


def run(
    *,
    timestamps_ns: NDArray[np.int64],
    close: NDArray[np.float64],
    dataset: DatasetMetadata,
    config: BacktestConfig,
    scenarios: List[IvScenario],
) -> BacktestResult:
    """Run the complete scenario set through exactly one PyO3 call."""
    validate_run_arrays(timestamps_ns, close)
    if not isinstance(dataset, DatasetMetadata):
        raise TypeError("dataset must be DatasetMetadata")
    if not isinstance(config, BacktestConfig):
        raise TypeError("config must be BacktestConfig")
    raw = _native.run_backtest(
        timestamps_ns,
        close,
        dataset.native(),
        config.native(),
        native_scenarios(scenarios),
    )
    return from_native(raw)

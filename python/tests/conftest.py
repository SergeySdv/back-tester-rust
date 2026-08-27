from __future__ import annotations

from typing import Tuple

import numpy as np
import pytest
from back_tester import BacktestConfig, DatasetMetadata


@pytest.fixture
def minute_arrays() -> Tuple[np.ndarray, np.ndarray]:
    timestamps = np.arange(2_881, dtype=np.int64) * 60_000_000_000
    close = np.full(2_881, 100.0, dtype=np.float64)
    return timestamps, close


@pytest.fixture
def metadata() -> DatasetMetadata:
    return DatasetMetadata("synthetic-two-days", "synthetic-fixture", "BTC-PROXY", 60, "UTC")


@pytest.fixture
def config() -> BacktestConfig:
    return BacktestConfig(
        initial_capital_usd=1_000.0,
        base_iv=0.55,
        margin_per_straddle_usd=100.0,
        quantity_step=0.1,
        risk_free_rate=0.01,
        carry_rate=0.005,
    )

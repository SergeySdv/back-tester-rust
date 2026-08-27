"""Python orchestration for the synthetic Black--Scholes scenario backtester."""

from ._native import price_many
from .config import BacktestConfig, DatasetMetadata, IvScenario
from .data import (
    OKX_HISTORY_CANDLES_MAPPING,
    ColumnMapping,
    MinuteData,
    load_minutes,
    load_okx_history_candles,
)
from .reporting import BacktestResult
from .runner import run

__all__ = [
    "BacktestConfig",
    "BacktestResult",
    "ColumnMapping",
    "DatasetMetadata",
    "IvScenario",
    "MinuteData",
    "OKX_HISTORY_CANDLES_MAPPING",
    "load_minutes",
    "load_okx_history_candles",
    "price_many",
    "run",
]

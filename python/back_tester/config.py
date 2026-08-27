"""Typed public configuration for the fixed MVP model."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from typing import Dict, List, Optional, Tuple, Union


@dataclass(frozen=True)
class DatasetMetadata:
    """Explicit source identity; no field is inferred from a filename."""

    dataset_id: str
    source: str
    symbol: str
    interval_seconds: int
    timezone: str

    def native(self) -> Dict[str, Union[str, int]]:
        """Return primitive values for the single native request."""
        return asdict(self)


@dataclass(frozen=True)
class BacktestConfig:
    """Effective inputs to the Rust-owned financial model."""

    initial_capital_usd: float
    base_iv: float
    margin_per_straddle_usd: float
    quantity_step: float
    risk_free_rate: float = 0.0
    carry_rate: float = 0.0

    def native(self) -> Dict[str, float]:
        """Return primitive values for the single native request."""
        return asdict(self)


@dataclass(frozen=True)
class IvScenario:
    """One of the three closed scenario variants supported by the MVP."""

    scenario_id: str
    shock_after_minutes: Optional[int]

    @classmethod
    def baseline(cls) -> IvScenario:
        return cls("baseline", None)

    @classmethod
    def stress_2x(cls, after_minutes: int) -> IvScenario:
        return cls("stress_2x", after_minutes)

    @classmethod
    def stress_3x(cls, after_minutes: int) -> IvScenario:
        return cls("stress_3x", after_minutes)

    def native(self) -> Tuple[str, Optional[int]]:
        return self.scenario_id, self.shock_after_minutes


def native_scenarios(scenarios: List[IvScenario]) -> List[Tuple[str, Optional[int]]]:
    """Translate typed variants without changing order or financial meaning."""
    if not isinstance(scenarios, list):
        raise TypeError("scenarios must be a list of IvScenario values")
    if any(not isinstance(item, IvScenario) for item in scenarios):
        raise TypeError("every scenario must be an IvScenario")
    return [scenario.native() for scenario in scenarios]

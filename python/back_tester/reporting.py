"""Lossless conversion from native column buffers to pandas tables."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional

import pandas as pd

EQUITY_DTYPES = {
    "timestamp_ns": "int64", "scenario_id": "string", "spot": "float64",
    "active_trade_id": "UInt64", "active_iv": "Float64", "cash": "float64",
    "option_liability": "float64", "locked_margin": "float64",
    "available_margin": "float64", "equity": "float64", "pnl": "float64",
    "running_peak": "float64", "drawdown_usd": "float64", "drawdown_pct": "float64",
    "margin_breached": "bool",
}
TRADES_DTYPES = {
    "scenario_id": "string", "trade_id": "uint64", "entry_timestamp_ns": "int64",
    "expiry_timestamp_ns": "int64", "strike": "float64", "entry_equity": "float64",
    "initial_quantity_steps": "uint64", "initial_quantity": "float64",
    "reserve_attempted": "bool", "reserve_executed_quantity_steps": "uint64",
    "reserve_executed_quantity": "float64", "total_received_premium": "float64",
    "settlement_spot": "float64", "total_expiry_payoff": "float64",
    "realized_pnl": "float64", "margin_breached_during_trade": "bool",
}
TRANCHES_DTYPES = {
    "scenario_id": "string", "trade_id": "uint64", "tranche_id": "uint64",
    "tranche_kind": "string", "execution_timestamp_ns": "int64",
    "expiry_timestamp_ns": "int64", "strike": "float64", "quantity_steps": "uint64",
    "quantity": "float64", "active_iv": "float64", "call_premium_per_unit": "float64",
    "put_premium_per_unit": "float64", "total_premium_per_unit": "float64",
    "received_premium": "float64", "locked_margin": "float64",
}
ATTEMPTS_DTYPES = {
    "scenario_id": "string", "trade_id": "uint64", "attempt_timestamp_ns": "int64",
    "requested_quantity_steps": "uint64", "executed_quantity_steps": "uint64",
    "requested_quantity": "float64", "executed_quantity": "float64",
    "available_margin_before": "float64", "reserve_budget_remaining": "float64",
    "outcome": "string", "reason": "string",
}
SUMMARY_DTYPES = {
    "scenario_id": "string", "terminal_status": "string", "terminal_timestamp_ns": "int64",
    "initial_equity": "float64", "final_equity": "float64", "total_pnl": "float64",
    "return_pct": "float64", "maximum_drawdown_usd": "float64",
    "maximum_drawdown_pct": "float64", "minimum_equity": "float64",
    "minimum_available_margin": "float64", "maximum_locked_margin": "float64",
    "completed_trade_count": "uint64", "reserve_attempt_count": "uint64",
    "full_reserve_count": "uint64", "reduced_reserve_count": "uint64",
    "rejected_reserve_count": "uint64", "skipped_incomplete_window_count": "uint8",
    "input_row_count": "uint64", "processed_row_count": "uint64",
    "ignored_input_row_count": "uint64", "any_margin_breach": "bool",
}


@dataclass(frozen=True)
class BacktestResult:
    equity_df: pd.DataFrame
    trades_df: pd.DataFrame
    tranches_df: pd.DataFrame
    reserve_attempts_df: pd.DataFrame
    summary_df: pd.DataFrame
    metadata: Mapping[str, Any]

    def export_csv(self, directory: Path) -> None:
        """Export the five audit tables and complete run metadata."""
        directory.mkdir(parents=True, exist_ok=True)
        tables = {
            "equity": self.equity_df,
            "trades": self.trades_df,
            "tranches": self.tranches_df,
            "reserve_attempts": self.reserve_attempts_df,
            "summary": self.summary_df,
        }
        for name, frame in tables.items():
            frame.to_csv(directory / f"{name}.csv", index=False)
        (directory / "metadata.json").write_text(
            json.dumps(dict(self.metadata), indent=2, sort_keys=True), encoding="utf-8"
        )


def from_native(raw: Mapping[str, Any]) -> BacktestResult:
    """Convert representation only; every financial value comes from Rust."""
    equity = _frame(
        raw["equity"], EQUITY_DTYPES,
        {"active_trade_id": "active_trade_id_valid", "active_iv": "active_iv_valid"},
    )
    return BacktestResult(
        equity_df=equity,
        trades_df=_frame(raw["trades"], TRADES_DTYPES),
        tranches_df=_frame(raw["tranches"], TRANCHES_DTYPES),
        reserve_attempts_df=_frame(raw["reserve_attempts"], ATTEMPTS_DTYPES),
        summary_df=_frame(raw["summary"], SUMMARY_DTYPES),
        metadata=dict(raw["metadata"]),
    )


def _frame(
    buffers: Mapping[str, List[Any]],
    dtypes: Mapping[str, str],
    nullable: Optional[Mapping[str, str]] = None,
) -> pd.DataFrame:
    nullable = nullable or {}
    columns: Dict[str, pd.Series] = {}
    for name, dtype in dtypes.items():
        array = pd.array(buffers[name], dtype=dtype)
        mask_name = nullable.get(name)
        if mask_name is not None:
            valid = pd.array(buffers[mask_name], dtype="bool").to_numpy()
            array[~valid] = pd.NA
        columns[name] = pd.Series(array, name=name)
    return pd.DataFrame(columns, columns=list(dtypes))

"""Run and reconcile the reviewed local OKX candle sample without downloading data."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any, Dict, List

from back_tester import (
    BacktestConfig,
    IvScenario,
    load_okx_history_candles,
    run,
)


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("sample", type=Path)
    parser.add_argument("--sha256", required=True)
    parser.add_argument("--output", type=Path)
    return parser.parse_args()


def _reconciliation(result: Any) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for summary in result.summary_df.to_dict(orient="records"):
        scenario_id = summary["scenario_id"]
        trades = result.trades_df[result.trades_df["scenario_id"] == scenario_id]
        equity = result.equity_df[result.equity_df["scenario_id"] == scenario_id]
        attempts = result.reserve_attempts_df[
            result.reserve_attempts_df["scenario_id"] == scenario_id
        ]
        trade_pnl = float(trades["realized_pnl"].sum())
        total_pnl = float(summary["total_pnl"])
        final_equity = float(equity.iloc[-1]["equity"])
        maximum_locked_margin = float(equity["locked_margin"].max())
        if len(trades) != summary["completed_trade_count"]:
            raise RuntimeError(f"trade count reconciliation failed for {scenario_id}")
        if not math.isclose(trade_pnl, total_pnl, rel_tol=1e-12, abs_tol=1e-9):
            raise RuntimeError(f"PnL reconciliation failed for {scenario_id}")
        if not math.isclose(final_equity, summary["final_equity"], abs_tol=1e-9):
            raise RuntimeError(f"final equity reconciliation failed for {scenario_id}")
        if not math.isclose(
            maximum_locked_margin, summary["maximum_locked_margin"], abs_tol=1e-9
        ):
            raise RuntimeError(f"locked margin reconciliation failed for {scenario_id}")
        if len(attempts) != summary["reserve_attempt_count"]:
            raise RuntimeError(f"reserve count reconciliation failed for {scenario_id}")
        if len(equity) != summary["processed_row_count"]:
            raise RuntimeError(f"equity row reconciliation failed for {scenario_id}")
        if not math.isclose(
            summary["initial_equity"] + total_pnl,
            summary["final_equity"],
            abs_tol=1e-9,
        ):
            raise RuntimeError(f"summary equity identity failed for {scenario_id}")
        rows.append({
            "scenario_id": scenario_id,
            "completed_trades": len(trades),
            "trade_pnl_sum": trade_pnl,
            "summary_total_pnl": total_pnl,
            "pnl_delta": trade_pnl - total_pnl,
            "equity_final": final_equity,
            "summary_final_equity": float(summary["final_equity"]),
            "equity_maximum_locked_margin": maximum_locked_margin,
            "summary_maximum_locked_margin": float(summary["maximum_locked_margin"]),
            "reserve_attempts": len(attempts),
            "equity_rows": len(equity),
        })
    return rows


def main() -> None:
    args = _arguments()
    minute_data = load_okx_history_candles(args.sample, expected_sha256=args.sha256)
    result = run(
        timestamps_ns=minute_data.timestamps_ns,
        close=minute_data.close,
        dataset=minute_data.metadata,
        config=BacktestConfig(1_000.0, 0.55, 100.0, 0.1, 0.0, 0.0),
        scenarios=[
            IvScenario.baseline(),
            IvScenario.stress_2x(720),
            IvScenario.stress_3x(720),
        ],
    )
    reconciliation = _reconciliation(result)
    if args.output is not None:
        result.export_csv(args.output)
    payload = {
        "metadata": dict(result.metadata),
        "input": {
            "row_count": len(minute_data.close),
            "start_timestamp_ns": int(minute_data.timestamps_ns[0]),
            "end_timestamp_ns": int(minute_data.timestamps_ns[-1]),
        },
        "table_counts": {
            "equity": len(result.equity_df),
            "trades": len(result.trades_df),
            "tranches": len(result.tranches_df),
            "reserve_attempts": len(result.reserve_attempts_df),
            "summary": len(result.summary_df),
        },
        "summary": json.loads(result.summary_df.to_json(orient="records")),
        "reconciliation": reconciliation,
    }
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()

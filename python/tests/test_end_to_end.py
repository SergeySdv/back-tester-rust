from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict

import pandas as pd
import pytest
from back_tester import BacktestConfig, DatasetMetadata, IvScenario, _native, run, runner
from back_tester.reporting import (
    ATTEMPTS_DTYPES,
    EQUITY_DTYPES,
    SUMMARY_DTYPES,
    TRADES_DTYPES,
    TRANCHES_DTYPES,
)


def scenarios() -> list[IvScenario]:
    return [
        IvScenario.stress_3x(720),
        IvScenario.baseline(),
        IvScenario.stress_2x(720),
    ]


def test_two_day_synthetic_run_reconciles_native_buffers(
    minute_arrays: tuple[Any, Any], metadata: DatasetMetadata, config: BacktestConfig
) -> None:
    timestamps, close = minute_arrays
    result = run(
        timestamps_ns=timestamps,
        close=close,
        dataset=metadata,
        config=config,
        scenarios=scenarios(),
    )
    raw = _native.run_backtest(
        timestamps, close, metadata.native(), config.native(),
        [scenario.native() for scenario in scenarios()],
    )

    assert result.summary_df["scenario_id"].tolist() == ["baseline", "stress_2x", "stress_3x"]
    assert result.summary_df["completed_trade_count"].tolist() == [2, 2, 2]
    assert len(result.trades_df) == 6
    assert len(result.equity_df) == 2_881 * 3
    assert result.summary_df["final_equity"].tolist() == raw["summary"]["final_equity"]
    assert result.summary_df["total_pnl"].tolist() == raw["summary"]["total_pnl"]
    assert result.summary_df["maximum_locked_margin"].tolist() == raw["summary"][
        "maximum_locked_margin"
    ]
    assert result.trades_df["realized_pnl"].sum() == pytest.approx(
        result.summary_df["total_pnl"].sum()
    )
    assert result.metadata["dataset_id"] == "synthetic-two-days"
    assert result.metadata["pricing_model"] == "synthetic Black-Scholes scenario backtest"
    assert result.metadata["risk_free_rate"] == 0.01
    assert result.metadata["carry_rate"] == 0.005
    assert result.metadata["scenarios"] == [
        ("baseline", 1.0, None), ("stress_2x", 2.0, 720), ("stress_3x", 3.0, 720)
    ]


def test_tables_have_exact_columns_dtypes_and_nullable_masks(
    minute_arrays: tuple[Any, Any], metadata: DatasetMetadata, config: BacktestConfig
) -> None:
    timestamps, close = minute_arrays
    result = run(
        timestamps_ns=timestamps,
        close=close,
        dataset=metadata,
        config=config,
        scenarios=[IvScenario.baseline(), IvScenario.stress_2x(1)],
    )
    expected: Dict[str, tuple[pd.DataFrame, Dict[str, str]]] = {
        "equity": (result.equity_df, EQUITY_DTYPES),
        "trades": (result.trades_df, TRADES_DTYPES),
        "tranches": (result.tranches_df, TRANCHES_DTYPES),
        "attempts": (result.reserve_attempts_df, ATTEMPTS_DTYPES),
        "summary": (result.summary_df, SUMMARY_DTYPES),
    }
    for frame, dtypes in expected.values():
        assert frame.columns.tolist() == list(dtypes)
        assert {name: str(dtype) for name, dtype in frame.dtypes.items()} == dtypes

    final_rows = result.equity_df.groupby("scenario_id", observed=True).tail(1)
    assert final_rows["active_trade_id"].isna().all()
    assert final_rows["active_iv"].isna().all()
    assert not result.equity_df["active_trade_id"].dropna().eq(2**64 - 1).any()
    assert not result.equity_df["active_iv"].dropna().isna().any()


def test_empty_reserve_attempt_table_preserves_contract_dtypes(
    minute_arrays: tuple[Any, Any], metadata: DatasetMetadata, config: BacktestConfig
) -> None:
    result = run(
        timestamps_ns=minute_arrays[0],
        close=minute_arrays[1],
        dataset=metadata,
        config=config,
        scenarios=[IvScenario.baseline()],
    )

    assert result.reserve_attempts_df.empty
    assert result.reserve_attempts_df.columns.tolist() == list(ATTEMPTS_DTYPES)
    assert {
        name: str(dtype) for name, dtype in result.reserve_attempts_df.dtypes.items()
    } == ATTEMPTS_DTYPES


def test_public_run_uses_exactly_one_native_bulk_call(
    monkeypatch: pytest.MonkeyPatch,
    minute_arrays: tuple[Any, Any],
    metadata: DatasetMetadata,
    config: BacktestConfig,
) -> None:
    calls = 0
    native_run = runner._native.run_backtest

    def counted(*args: Any, **kwargs: Any) -> Any:
        nonlocal calls
        calls += 1
        return native_run(*args, **kwargs)

    monkeypatch.setattr(runner._native, "run_backtest", counted)
    run(
        timestamps_ns=minute_arrays[0], close=minute_arrays[1], dataset=metadata,
        config=config, scenarios=[IvScenario.baseline()],
    )
    assert calls == 1


def test_repeat_is_deterministic(
    minute_arrays: tuple[Any, Any], metadata: DatasetMetadata, config: BacktestConfig
) -> None:
    kwargs = dict(
        timestamps_ns=minute_arrays[0], close=minute_arrays[1], dataset=metadata,
        config=config, scenarios=scenarios(),
    )
    first = run(**kwargs)
    second = run(**kwargs)
    pd.testing.assert_frame_equal(first.equity_df, second.equity_df, check_exact=True)
    pd.testing.assert_frame_equal(first.trades_df, second.trades_df, check_exact=True)
    pd.testing.assert_frame_equal(first.tranches_df, second.tranches_df, check_exact=True)
    pd.testing.assert_frame_equal(first.reserve_attempts_df, second.reserve_attempts_df, check_exact=True)
    pd.testing.assert_frame_equal(first.summary_df, second.summary_df, check_exact=True)
    assert first.metadata == second.metadata


def test_native_typed_error_has_field_and_no_partial_result(
    minute_arrays: tuple[Any, Any], metadata: DatasetMetadata, config: BacktestConfig
) -> None:
    invalid = BacktestConfig(0.0, config.base_iv, config.margin_per_straddle_usd, config.quantity_step)
    with pytest.raises(ValueError, match=r"config.initial_capital_usd.*greater than zero"):
        run(
            timestamps_ns=minute_arrays[0], close=minute_arrays[1], dataset=metadata,
            config=invalid, scenarios=[IvScenario.baseline()],
        )


def test_fixed_scenario_validation_is_native_and_actionable(
    minute_arrays: tuple[Any, Any], metadata: DatasetMetadata, config: BacktestConfig
) -> None:
    with pytest.raises(ValueError, match=r"scenario.scenario_id.*unsupported"):
        run(
            timestamps_ns=minute_arrays[0], close=minute_arrays[1], dataset=metadata,
            config=config, scenarios=[IvScenario("custom", None)],
        )
    with pytest.raises(ValueError, match="scenarios.*must not be empty"):
        run(
            timestamps_ns=minute_arrays[0], close=minute_arrays[1], dataset=metadata,
            config=config, scenarios=[],
        )


@pytest.mark.parametrize(
    ("dataset_factory", "scenario_factory", "field", "maximum"),
    [
        (
            lambda metadata: metadata,
            lambda: [IvScenario.stress_2x(-1)],
            "shock_after_minutes",
            65_535,
        ),
        (
            lambda metadata: metadata,
            lambda: [IvScenario.stress_2x(65_536)],
            "shock_after_minutes",
            65_535,
        ),
        (
            lambda metadata: metadata,
            lambda: [IvScenario.stress_2x(10**100)],
            "shock_after_minutes",
            65_535,
        ),
        (
            lambda metadata: DatasetMetadata(
                metadata.dataset_id,
                metadata.source,
                metadata.symbol,
                -1,
                metadata.timezone,
            ),
            lambda: [IvScenario.baseline()],
            "interval_seconds",
            4_294_967_295,
        ),
        (
            lambda metadata: DatasetMetadata(
                metadata.dataset_id,
                metadata.source,
                metadata.symbol,
                10**100,
                metadata.timezone,
            ),
            lambda: [IvScenario.baseline()],
            "interval_seconds",
            4_294_967_295,
        ),
        (
            lambda metadata: DatasetMetadata(
                metadata.dataset_id,
                metadata.source,
                metadata.symbol,
                4_294_967_296,
                metadata.timezone,
            ),
            lambda: [IvScenario.baseline()],
            "interval_seconds",
            4_294_967_295,
        ),
    ],
)
def test_unsigned_native_fields_report_the_invalid_field(
    minute_arrays: tuple[Any, Any],
    metadata: DatasetMetadata,
    config: BacktestConfig,
    dataset_factory: Any,
    scenario_factory: Any,
    field: str,
    maximum: int,
) -> None:
    with pytest.raises(ValueError, match=rf"{field}.*0\.\.={maximum}"):
        run(
            timestamps_ns=minute_arrays[0],
            close=minute_arrays[1],
            dataset=dataset_factory(metadata),
            config=config,
            scenarios=scenario_factory(),
        )


def test_invalid_public_arrays_are_rejected_before_the_native_call(
    monkeypatch: pytest.MonkeyPatch,
    minute_arrays: tuple[Any, Any],
    metadata: DatasetMetadata,
    config: BacktestConfig,
) -> None:
    def unexpected_native_call(*args: Any, **kwargs: Any) -> Any:
        raise AssertionError("native call must not run for an invalid public array")

    monkeypatch.setattr(runner._native, "run_backtest", unexpected_native_call)
    with pytest.raises(ValueError, match="timestamps_ns must be C-contiguous"):
        run(
            timestamps_ns=minute_arrays[0][::2],
            close=minute_arrays[1][:1_441],
            dataset=metadata,
            config=config,
            scenarios=[IvScenario.baseline()],
        )


def test_audit_export_contains_five_tables_and_honest_metadata(
    tmp_path: Path,
    minute_arrays: tuple[Any, Any],
    metadata: DatasetMetadata,
    config: BacktestConfig,
) -> None:
    result = run(
        timestamps_ns=minute_arrays[0], close=minute_arrays[1], dataset=metadata,
        config=config, scenarios=[IvScenario.baseline()],
    )
    result.export_csv(tmp_path)
    assert {path.name for path in tmp_path.iterdir()} == {
        "equity.csv", "trades.csv", "tranches.csv", "reserve_attempts.csv",
        "summary.csv", "metadata.json",
    }
    exported = json.loads((tmp_path / "metadata.json").read_text(encoding="utf-8"))
    assert exported["pricing_model"] == "synthetic Black-Scholes scenario backtest"

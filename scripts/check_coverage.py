#!/usr/bin/env python3
"""Enforce independent Rust/Python line and branch coverage thresholds."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path
from typing import Any


class CoverageReportError(ValueError):
    """A required coverage report is missing, malformed, or incomplete."""


def _read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise CoverageReportError(f"missing coverage report: {path}") from error
    except (OSError, json.JSONDecodeError) as error:
        raise CoverageReportError(f"invalid coverage report {path}: {error}") from error
    if not isinstance(value, dict):
        raise CoverageReportError(f"invalid coverage report {path}: root must be an object")
    return value


def _percentage(value: Any, label: str) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise CoverageReportError(f"missing or invalid percentage: {label}")
    result = float(value)
    if not math.isfinite(result) or not 0.0 <= result <= 100.0:
        raise CoverageReportError(f"percentage outside [0, 100]: {label}")
    return result


def rust_percentage(path: Path, metric: str) -> float:
    report = _read_json(path)
    try:
        data = report["data"]
        if not isinstance(data, list) or len(data) != 1:
            raise KeyError("data")
        value = data[0]["totals"][metric]["percent"]
    except (KeyError, TypeError, IndexError) as error:
        raise CoverageReportError(f"missing Rust {metric} coverage in {path}") from error
    return _percentage(value, f"Rust {metric}")


def python_percentages(path: Path) -> tuple[float, float]:
    report = _read_json(path)
    try:
        totals = report["totals"]
        lines = totals["percent_statements_covered"]
        branch_total = totals["num_branches"]
        branch_covered = totals["covered_branches"]
    except (KeyError, TypeError) as error:
        raise CoverageReportError(f"missing Python coverage totals in {path}") from error
    if not isinstance(branch_total, int) or isinstance(branch_total, bool) or branch_total < 0:
        raise CoverageReportError(f"invalid Python branch count: {path}")
    if not isinstance(branch_covered, int) or isinstance(branch_covered, bool) or not 0 <= branch_covered <= branch_total:
        raise CoverageReportError(f"invalid Python covered branch count: {path}")
    branch_percentage = 100.0 if branch_total == 0 else 100.0 * branch_covered / branch_total
    return _percentage(lines, "Python lines"), branch_percentage


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust-lines", type=Path, required=True)
    parser.add_argument("--min-rust-lines", type=float, required=True)
    parser.add_argument("--rust-branches", type=Path, required=True)
    parser.add_argument("--min-rust-branches", type=float, required=True)
    parser.add_argument("--python", type=Path, required=True)
    parser.add_argument("--min-python-lines", type=float, required=True)
    parser.add_argument("--min-python-branches", type=float, required=True)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        required = {
            "Rust lines": _percentage(args.min_rust_lines, "minimum Rust lines"),
            "Rust branches": _percentage(args.min_rust_branches, "minimum Rust branches"),
            "Python lines": _percentage(args.min_python_lines, "minimum Python lines"),
            "Python branches": _percentage(args.min_python_branches, "minimum Python branches"),
        }
        actual = {
            "Rust lines": rust_percentage(args.rust_lines, "lines"),
            "Rust branches": rust_percentage(args.rust_branches, "branches"),
        }
        python_lines, python_branches = python_percentages(args.python)
        actual.update({"Python lines": python_lines, "Python branches": python_branches})
    except CoverageReportError as error:
        print(f"coverage gate error: {error}")
        return 2

    failed = False
    for label, minimum in required.items():
        value = actual[label]
        print(f"{label}: {value:.2f}% (required {minimum:.2f}%)")
        failed |= value < minimum
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())

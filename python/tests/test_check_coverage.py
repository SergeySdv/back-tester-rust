import json
import subprocess
import sys
from pathlib import Path

import pytest


SCRIPT = Path(__file__).parents[2] / "scripts" / "check_coverage.py"


def _write_reports(tmp_path: Path, rust_lines: float = 95.0, rust_branches: float = 90.0, python_lines: float = 92.0, covered_branches: int = 9) -> tuple[Path, Path, Path]:
    lines = tmp_path / "rust-lines.json"
    branches = tmp_path / "rust-branches.json"
    python = tmp_path / "python.json"
    lines.write_text(json.dumps({"data": [{"totals": {"lines": {"percent": rust_lines}}}]}))
    branches.write_text(json.dumps({"data": [{"totals": {"branches": {"percent": rust_branches}}}]}))
    python.write_text(json.dumps({"totals": {"percent_covered": python_lines, "percent_statements_covered": python_lines, "num_branches": 10, "covered_branches": covered_branches}}))
    return lines, branches, python


def _run(reports: tuple[Path, Path, Path]) -> subprocess.CompletedProcess[str]:
    lines, branches, python = reports
    return subprocess.run([sys.executable, str(SCRIPT), "--rust-lines", str(lines), "--min-rust-lines", "90", "--rust-branches", str(branches), "--min-rust-branches", "85", "--python", str(python), "--min-python-lines", "85", "--min-python-branches", "80"], text=True, capture_output=True, check=False)


def test_threshold_checker_accepts_reports_above_all_thresholds(tmp_path: Path) -> None:
    result = _run(_write_reports(tmp_path))
    assert result.returncode == 0
    assert "Rust lines: 95.00%" in result.stdout


@pytest.mark.parametrize("changes", [{"rust_lines": 89.9}, {"rust_branches": 84.9}, {"python_lines": 84.9}, {"covered_branches": 7}])
def test_threshold_checker_rejects_each_below_threshold_metric(tmp_path: Path, changes: dict[str, float]) -> None:
    result = _run(_write_reports(tmp_path, **changes))
    assert result.returncode == 1


def test_threshold_checker_uses_python_statement_coverage_for_line_gate(tmp_path: Path) -> None:
    reports = _write_reports(tmp_path, python_lines=90.0, covered_branches=10)
    python_report = json.loads(reports[2].read_text())
    python_report["totals"]["percent_statements_covered"] = 80.0
    reports[2].write_text(json.dumps(python_report))

    assert _run(reports).returncode == 1


@pytest.mark.parametrize("contents", ["not-json", "[]", '{}'])
def test_threshold_checker_rejects_malformed_reports(tmp_path: Path, contents: str) -> None:
    reports = _write_reports(tmp_path)
    reports[0].write_text(contents)
    assert _run(reports).returncode == 2


def test_threshold_checker_rejects_missing_report(tmp_path: Path) -> None:
    reports = _write_reports(tmp_path)
    reports[1].unlink()
    assert _run(reports).returncode == 2

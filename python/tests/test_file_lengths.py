import subprocess
import sys
from pathlib import Path

import pytest

SCRIPT = Path(__file__).parents[2] / "scripts" / "check_file_lengths.py"


def _write(root: Path, relative: str, line_count: int) -> Path:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("line\n" * line_count)
    return path


def _repository(tmp_path: Path) -> Path:
    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    return tmp_path


def _run(root: Path, limit: int = 500) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), "--root", str(root), "--limit", str(limit)],
        text=True,
        capture_output=True,
        check=False,
    )


def _track(root: Path) -> None:
    subprocess.run(["git", "-C", str(root), "add", "."], check=True)


def test_file_at_exact_limit_passes(tmp_path: Path) -> None:
    root = _repository(tmp_path)
    _write(root, "crates/example/src/lib.rs", 500)
    _track(root)

    assert _run(root).returncode == 0


def test_file_above_limit_has_actionable_error(tmp_path: Path) -> None:
    root = _repository(tmp_path)
    _write(root, "python/package/module.py", 501)
    _track(root)

    result = _run(root)

    assert result.returncode == 1
    assert "python/package/module.py: 501 physical lines (limit 500)" in result.stderr


@pytest.mark.parametrize("suffix", [".md", ".toml", ".json", ".pyc"])
def test_non_executable_extensions_are_ignored(tmp_path: Path, suffix: str) -> None:
    root = _repository(tmp_path)
    _write(root, f"scripts/large{suffix}", 501)
    _track(root)

    assert _run(root).returncode == 0


@pytest.mark.parametrize(
    "directory", ["target", ".venv", "vendor", "build", "dist", "__pycache__"]
)
def test_generated_and_vendor_directories_are_excluded(
    tmp_path: Path, directory: str
) -> None:
    root = _repository(tmp_path)
    _write(root, f"python/{directory}/generated.py", 501)
    _track(root)

    assert _run(root).returncode == 0


def test_new_source_files_are_discovered_without_an_allowlist(tmp_path: Path) -> None:
    root = _repository(tmp_path)
    _write(root, "scripts/future_tool.sh", 501)

    assert _run(root).returncode == 1


def test_tracked_source_cannot_be_hidden_by_a_later_gitignore_rule(
    tmp_path: Path,
) -> None:
    root = _repository(tmp_path)
    _write(root, "python/package/hidden.py", 501)
    _track(root)
    (root / ".gitignore").write_text("python/package/hidden.py\n")

    result = _run(root)

    assert result.returncode == 1
    assert "python/package/hidden.py: 501 physical lines (limit 500)" in result.stderr

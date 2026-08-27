"""Enforce the repository's physical-line limit for executable source files."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

LINE_LIMIT = 500
SOURCE_ROOTS = ("crates", "python", "scripts")
SOURCE_SUFFIXES = frozenset({".py", ".rs", ".sh"})
EXCLUDED_DIRECTORY_NAMES = frozenset(
    {".venv", "__pycache__", "build", "dist", "target", "vendor"}
)


def _candidate_paths(root: Path) -> list[Path]:
    command = [
        "git",
        "-C",
        str(root),
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "-z",
        "--",
        *SOURCE_ROOTS,
    ]
    completed = subprocess.run(command, capture_output=True, check=False)
    if completed.returncode != 0:
        message = completed.stderr.decode(errors="replace").strip()
        raise RuntimeError(f"unable to discover source files with git: {message}")
    return [root / Path(raw.decode()) for raw in completed.stdout.split(b"\0") if raw]


def checked_source_paths(root: Path) -> list[Path]:
    """Return automatically discovered, non-generated executable sources."""
    paths = []
    for path in _candidate_paths(root):
        relative = path.relative_to(root)
        if path.suffix not in SOURCE_SUFFIXES:
            continue
        if any(part in EXCLUDED_DIRECTORY_NAMES for part in relative.parts[:-1]):
            continue
        paths.append(path)
    return sorted(paths)


def physical_line_count(path: Path) -> int:
    """Count physical lines consistently for files with or without a final newline."""
    return len(path.read_bytes().splitlines())


def violations(root: Path, limit: int = LINE_LIMIT) -> list[tuple[Path, int]]:
    return [
        (path.relative_to(root), count)
        for path in checked_source_paths(root)
        if (count := physical_line_count(path)) > limit
    ]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--limit", type=int, default=LINE_LIMIT)
    args = parser.parse_args(argv)
    if args.limit < 1:
        parser.error("--limit must be positive")

    root = args.root.resolve()
    try:
        failures = violations(root, args.limit)
    except (OSError, RuntimeError) as error:
        print(f"file-length check failed: {error}", file=sys.stderr)
        return 2
    if failures:
        for path, count in failures:
            print(f"{path}: {count} physical lines (limit {args.limit})", file=sys.stderr)
        return 1

    print(f"file-length check passed: all executable sources are <= {args.limit} lines")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

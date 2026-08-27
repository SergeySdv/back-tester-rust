import shlex
from pathlib import Path

QUALITY_SCRIPT = Path(__file__).parents[2] / "scripts" / "quality.sh"


def test_ruff_gate_covers_every_executable_source_root() -> None:
    commands = [
        shlex.split(line)
        for line in QUALITY_SCRIPT.read_text().splitlines()
        if line.startswith("uv run ruff check ")
    ]

    assert len(commands) == 1
    targets = set(commands[0][4:])
    assert "." in targets or {"crates", "python", "scripts"} <= targets

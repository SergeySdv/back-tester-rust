import math

import pytest
from back_tester import price_many


def test_native_bulk_pricing_is_importable_and_deterministic() -> None:
    args = ([100.0, 120.0], [100.0, 100.0], [1.0, 0.0], [0.2, 0.3], 0.05, 0.01)
    first = price_many(*args)
    second = price_many(*args)
    assert first == second
    assert first[0][1] == 20.0
    assert first[1][1] == 0.0
    assert all(math.isfinite(value) for side in first for value in side)


def test_native_error_is_actionable_python_exception() -> None:
    with pytest.raises(ValueError, match=r"invalid field `spot`: must be greater than zero"):
        price_many([0.0], [100.0], [1.0], [0.2], 0.0, 0.0)


def test_native_rejects_mismatched_bulk_lengths() -> None:
    with pytest.raises(ValueError, match=r"length mismatch for `strike`: expected 1, got 0"):
        price_many([100.0], [], [1.0], [0.2], 0.0, 0.0)

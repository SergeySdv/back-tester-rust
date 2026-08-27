"""Thin Python access to deterministic Rust pricing."""

from ._native import price_many

__all__ = ["price_many"]

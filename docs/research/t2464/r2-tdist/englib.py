"""r2-tdist: Excel functions the T2464 harness does not expose (ERFC.PRECISE, SQRT, ...), evaluated through the
engine's python bindings (`import rustsheet`), one workbook per batch. Inputs are exact doubles set as numbers."""
from __future__ import annotations

import rustsheet as ic

_CACHE: dict = {}


def eng(fn: str, xs: list[float]) -> list[float]:
    """Engine value of =fn(x) for each double x (fn like 'ERFC.PRECISE'); cached."""
    todo = sorted({x for x in xs if (fn, x) not in _CACHE})
    for k in range(0, len(todo), 20000):
        chunk = todo[k:k + 20000]
        m = ic.create("w", "en", "UTC")
        for i, x in enumerate(chunk):
            m.update_cell_with_number(0, i + 1, 1, x)
            m.update_cell_with_formula(0, i + 1, 2, f"={fn}(A{i + 1})")
        m.evaluate()
        for i, x in enumerate(chunk):
            _CACHE[(fn, x)] = float(m.get_cell_value(0, i + 1, 2) if not hasattr(m, "get_cell_value") else
                                    _num(m.get_cell_value(0, i + 1, 2)))
    return [_CACHE[(fn, x)] for x in xs]


def _num(v):
    if isinstance(v, (int, float)):
        return v
    if isinstance(v, tuple):
        return v[-1]
    return float(v)


MISSING: set = set()


def eng_lazy(fn: str, x: float, placeholder: float) -> float:
    """Cached engine value, or record the miss and return `placeholder` (call fetch_missing() and re-run)."""
    v = _CACHE.get((fn, x))
    if v is None:
        MISSING.add((fn, x))
        return placeholder
    return v


def fetch_missing() -> int:
    by: dict = {}
    for fn, x in MISSING:
        by.setdefault(fn, []).append(x)
    for fn, xs in by.items():
        eng(fn, xs)
    k = len(MISSING)
    MISSING.clear()
    return k

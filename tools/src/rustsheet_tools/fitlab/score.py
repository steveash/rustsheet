"""Scoring candidates against a sweep: bit-exact, within 4 ulp, worst cells, ranking.

The verdict boundaries are the harness's (crates/rustsheet-eval/src/value.rs `compare_numbers`):
equal doubles (0.0 == -0.0) are exact, finite pairs within 4 ulp (`ulp_distance`) are `ulp`,
pairs within 1e-9 relative are `close`, anything else (including a candidate that raises or
returns a non-number) is a miss. A cell whose Excel value is an error is matched by the same error:
a candidate signals one by raising `ExcelError` (or returning it); the right error is exact, another
error or a number is a miss (T-1534).
"""

from __future__ import annotations

import math
import struct
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from rustsheet_tools.fitlab.sweep import Cell

ULP_TOLERANCE = 4  # value.rs: `ulp_distance(e, a) <= 4` is NumericUlp
CLOSE_RELATIVE = 1e-9  # value.rs: NumericClose
_I64_MIN = -(1 << 63)


def _ordered(x: float) -> int:
    """The IEEE-754 bit pattern on a monotonically ordered integer line (value.rs `ordered`)."""
    (bits,) = struct.unpack("<q", struct.pack("<d", x))
    return _I64_MIN - bits if bits < 0 else int(bits)


def ulp_distance(a: float, b: float) -> int:
    """Distance between two finite doubles in units in the last place, exactly as the harness
    computes it (value.rs `ulp_distance`): 0.0 and -0.0 are 0 apart, ±min-subnormal 2 apart."""
    return abs(_ordered(a) - _ordered(b))


class ExcelError(ArithmeticError):
    """A candidate's Excel error result (`#NUM!` unless `code` says otherwise)."""

    def __init__(self, message: str = "", code: str = "#NUM!") -> None:
        super().__init__(message or code)
        self.code = code


@dataclass(frozen=True)
class Candidate:
    """A named implementation: `evaluate(cell)` returns the double it computes for the cell.

    Build one with `candidate()` from a function of the cell's arguments; the engine's
    pseudo-candidate (fitlab.engine) looks values up by cell instead. `family` groups variants of
    one idea (e.g. one continued fraction at many depths) for the per-family summary."""

    name: str
    evaluate: Callable[[Cell], float]
    family: str = ""
    # Optional: the iterates the candidate walks through on the cell, in order (it may stop early
    # by raising). The score reports which one equals Excel's value.
    trace: Callable[[Cell], list[float]] | None = None


def candidate(
    name: str,
    fn: Callable[..., float],
    family: str = "",
    trace: Callable[..., list[float]] | None = None,
) -> Candidate:
    """A candidate that calls `fn(*cell.args)`; `trace`, if given, is called the same way."""
    traced = None if trace is None else (lambda cell: trace(*cell.args))
    return Candidate(name, lambda cell: fn(*cell.args), family or name, traced)


@dataclass
class CellResult:
    cell: Cell
    actual: float | None  # None: the candidate raised or returned a non-number
    ulps: int | None  # None: not comparable (non-finite or no value)
    note: str = ""
    error: str | None = None  # the Excel error the candidate produced, if it did
    iterate: int | None = None  # index of the first trace iterate equal to Excel's value

    def to_json(self) -> dict[str, Any]:
        return {
            "case": self.cell.case,
            "cell": self.cell.key,
            "formula": self.cell.formula,
            "expected": self.cell.error if self.cell.error is not None else self.cell.expected,
            "actual": self.error if self.error is not None else self.actual,
            "ulps": self.ulps,
            "note": self.note,
            "iterate": self.iterate,
        }


@dataclass
class Score:
    """One candidate's result over a sweep."""

    name: str
    family: str
    total: int
    exact: int = 0
    ulp: int = 0  # within 4 ulp, exact included
    close: int = 0  # within 1e-9 relative, ulp included
    errors: int = 0
    exact_keys: set[str] = field(default_factory=set)
    traced: int = 0  # cells where the candidate exposed iterates
    iterate_hits: dict[str, int] = field(default_factory=dict)  # "case cell" -> matching iterate
    incomparable: int = 0  # cells with no finite distance (errors, non-finite values)
    largest: int = 0  # the largest finite ulp distance
    results: list[CellResult] = field(default_factory=list)  # the worst cells only (see `score`)

    @property
    def max_ulps(self) -> int | None:
        """The largest ulp distance, or None when some cell has no comparable value."""
        return None if self.incomparable else self.largest

    def sort_key(self) -> tuple[int, int, int, float]:
        worst = self.max_ulps
        return (-self.exact, -self.ulp, -self.close, math.inf if worst is None else worst)

    def worst(self, k: int) -> list[CellResult]:
        """The `k` cells furthest from Excel (no value first, then by ulp distance)."""
        misses = [r for r in self.results if r.ulps != 0]
        misses.sort(key=lambda r: (r.ulps is not None, -(r.ulps or 0)))
        return misses[:k]

    def to_json(self, worst: int) -> dict[str, Any]:
        return {
            "name": self.name,
            "family": self.family,
            "cells": self.total,
            "exact": self.exact,
            "within_4_ulp": self.ulp,
            "within_1e-9": self.close,
            "errors": self.errors,
            "traced": self.traced,
            "iterate_hits": self.iterate_hits,
            "max_ulps": self.max_ulps,
            "worst": [r.to_json() for r in self.worst(worst)],
        }


def _measure(cell: Cell, cand: Candidate) -> CellResult:
    try:
        value = cand.evaluate(cell)
    except ExcelError as e:  # before the ArithmeticError clause: ExcelError is one
        return _error_result(cell, e.code)
    except (ArithmeticError, ValueError, TypeError, KeyError, IndexError, RecursionError) as e:
        return CellResult(cell, None, None, f"{type(e).__name__}: {e}")
    if isinstance(value, bool) or not isinstance(value, float | int):
        return CellResult(cell, None, None, f"returned {type(value).__name__}")
    return _number_result(cell, float(value))


def _number_result(cell: Cell, actual: float) -> CellResult:
    expected = cell.expected
    if cell.error is not None:
        return CellResult(cell, actual, None, f"number instead of {cell.error}")
    if actual == expected or (math.isnan(actual) and math.isnan(expected)):
        return CellResult(cell, actual, 0)
    if not (math.isfinite(actual) and math.isfinite(expected)):
        return CellResult(cell, actual, None, "non-finite")
    return CellResult(cell, actual, ulp_distance(expected, actual))


def _error_result(cell: Cell, code: str) -> CellResult:
    """The candidate's error against Excel's value: the same error is exact (0 ulp), anything else
    a miss; an error where Excel has a number is a miss too."""
    if cell.error is None:
        return CellResult(cell, None, None, f"returned {code}", code)
    if code == cell.error:
        return CellResult(cell, None, 0, "", code)
    return CellResult(cell, None, None, f"{code} instead of {cell.error}", code)


def _iterate(cell: Cell, cand: Candidate) -> int | None:
    """Index of the first iterate of the candidate's trace that equals Excel's value."""
    if cand.trace is None or cell.error is not None:
        return None
    try:
        steps = cand.trace(cell)
    except (ArithmeticError, ValueError, TypeError, KeyError, IndexError, RecursionError):
        return None
    return next((i for i, v in enumerate(steps) if v == cell.expected), None)


def score(cells: list[Cell], cand: Candidate, keep: int = 10) -> Score:
    """Runs `cand` on every cell; a candidate that raises on a cell scores a miss there. Only the
    `keep` worst cells are kept, so scoring thousands of candidates stays small."""
    s = Score(cand.name, cand.family or cand.name, len(cells))
    for cell in cells:
        r = _measure(cell, cand)
        if cand.trace is not None and cell.error is None:
            s.traced += 1
            r.iterate = _iterate(cell, cand)
            if r.iterate is not None:
                s.iterate_hits[f"{cell.case} {cell.key}"] = r.iterate
        s.results.append(r)
        if r.actual is None and r.error is None:
            s.errors += 1
        if r.ulps is None:
            s.incomparable += 1
            continue
        s.largest = max(s.largest, r.ulps)
        if r.ulps == 0:
            s.exact += 1
            s.exact_keys.add(f"{cell.case} {cell.key}")
        if r.ulps <= ULP_TOLERANCE:
            s.ulp += 1
        if r.error is not None or (
            r.actual is not None
            and abs(cell.expected - r.actual) <= CLOSE_RELATIVE * max(abs(cell.expected), abs(r.actual))
        ):
            s.close += 1
    s.results = s.worst(keep)
    return s


def rank(scores: list[Score]) -> list[Score]:
    """Best first: most bit-exact, then most within 4 ulp, within 1e-9, smallest worst ulp."""
    return sorted(scores, key=Score.sort_key)


def _pct(part: int, whole: int) -> str:
    return f"{100 * part / whole:5.1f}%" if whole else "    -"


def _ulps_text(n: int | None) -> str:
    return "-" if n is None else f"{n:,}"


def table(scores: list[Score], top: int) -> str:
    """The ranked table (the first `top` rows, plus any pinned rows such as the engine's)."""
    ranked = rank(scores)
    lines = [
        f"{'#':>4}  {'candidate':<44} {'exact':>13} {'<=4 ulp':>13} {'<=1e-9':>13} {'err':>5} {'max ulp':>9}",
    ]
    for i, s in enumerate(ranked, 1):
        if i > top and not s.name.startswith("engine"):
            continue
        lines.append(
            f"{i:>4}  {s.name[:44]:<44} {s.exact:>6} {_pct(s.exact, s.total)} {s.ulp:>6} "
            f"{_pct(s.ulp, s.total)} {s.close:>6} {_pct(s.close, s.total)} {s.errors:>5} "
            f"{_ulps_text(s.max_ulps):>9}"
        )
    return "\n".join(lines)


def families(scores: list[Score]) -> list[dict[str, Any]]:
    """Per family, in first-seen order: its best member and the cumulative union of bit-exact
    cells (how many cells some candidate so far gets exactly; decision 0056 records both)."""
    order: dict[str, list[Score]] = {}
    for s in scores:
        order.setdefault(s.family, []).append(s)
    union: set[str] = set()
    out = []
    for name, members in order.items():
        best = rank(members)[0]
        own: set[str] = set().union(*(m.exact_keys for m in members))
        union |= own
        out.append(
            {
                "family": name,
                "candidates": len(members),
                "best": best.name,
                "best_exact": best.exact,
                "family_union_exact": len(own),
                "cumulative_union_exact": len(union),
            }
        )
    return out


def family_table(rows: list[dict[str, Any]], total: int) -> str:
    lines = [f"{'family':<34} {'members':>7} {'best':>6} {'union':>6} {'cumulative':>10}  best member"]
    lines += [
        f"{r['family'][:34]:<34} {r['candidates']:>7} {r['best_exact']:>6} {r['family_union_exact']:>6} "
        f"{r['cumulative_union_exact']:>6}/{total:<4} {r['best']}"
        for r in rows
    ]
    return "\n".join(lines)


def worst_text(s: Score, k: int) -> str:
    lines = [f"worst cells of {s.name}:"]
    for r in s.worst(k):
        got = r.error or ("-" if r.actual is None else repr(r.actual))
        detail = f"{_ulps_text(r.ulps)} ulp" if r.ulps is not None else r.note
        where = f"{r.cell.case.rsplit('/', 1)[-1].removesuffix('.case.json')} {r.cell.key}"
        lines.append(
            f"  {where:<26} {r.cell.formula:<26} excel {r.cell.error or r.cell.expected!r}  got {got}  ({detail})"
        )
    return "\n".join(lines)


def iterate_text(scores: list[Score]) -> str:
    """Per traced candidate: on how many cells Excel's value is one of its iterates, and the
    spread of the matching iterate's index (the bisection depth, for a bisection)."""
    lines = []
    for s in rank(scores):
        if not s.traced:
            continue
        depths = sorted(s.iterate_hits.values())
        spread = f", index {depths[0]}..{depths[-1]} (median {depths[len(depths) // 2]})" if depths else ""
        lines.append(f"  {s.name[:60]:<60} {len(depths):>5}/{s.traced:<5} cells{spread}")
    return "\n".join(["iterate matches (Excel's value is iterate k of the trace):", *lines]) if lines else ""

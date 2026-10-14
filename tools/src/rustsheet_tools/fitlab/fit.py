"""A fit: one kernel's sweep, its candidates, and the run that scores and ranks them."""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from rustsheet_tools.fitlab.score import Candidate, ExcelError, Score, families, rank, score
from rustsheet_tools.fitlab.sweep import Cell, Sweep, load_sweep
from rustsheet_tools.snapshot import show_cells

ENGINE = "engine (today)"


@dataclass
class Fit:
    """What to score: `cases` (`suite/name`), the formula `pattern` whose first group is the
    argument list, the candidates, and an optional `where(cell)` filter (e.g. a sub-domain)."""

    name: str
    description: str
    cases: list[str]
    pattern: str
    candidates: Callable[[], list[Candidate]]  # built lazily: some families are large
    where: Callable[[Cell], bool] | None = None
    errors: bool = False  # also score the cells whose Excel value is an error
    notes: list[str] = field(default_factory=list)


def engine_candidate(case_ids: list[str]) -> Candidate:
    """Today's engine as a pseudo-candidate: its value per cell from `rustsheet-eval show --all`
    (a release build of the harness; see snapshot.eval_binary)."""
    actual: dict[str, str] = {}
    for case_id in case_ids:
        for key, shown in show_cells(case_id).items():
            actual[f"{case_id} {key}"] = shown.get("actual", "")

    def evaluate(cell: Cell) -> float:
        text = actual.get(f"{cell.case} {cell.key}")
        if text is None:
            msg = f"the engine did not score {cell.key}"
            raise KeyError(msg)
        if text.startswith("#"):
            raise ExcelError(code=text)
        try:
            return float(text)  # Rust's `{:?}` of an f64 round-trips exactly
        except ValueError:
            msg = f"engine value {text}"
            raise ValueError(msg) from None

    return Candidate(ENGINE, evaluate, "engine")


@dataclass
class FitResult:
    fit: Fit
    sweep: Sweep
    scores: list[Score]

    def to_json(self, worst: int) -> dict[str, Any]:
        return {
            "fit": self.fit.name,
            "description": self.fit.description,
            "cases": self.fit.cases,
            "pattern": self.fit.pattern,
            "cells": len(self.sweep.cells),
            "unparsed": self.sweep.unparsed,
            "non_numeric": self.sweep.non_numeric,
            "excluded": self.sweep.excluded,
            "candidates": [s.to_json(worst) for s in rank(self.scores)],
            "families": families(self.scores),
        }


def run(fit: Fit, *, engine: bool = True, only: str | None = None, keep: int = 10) -> FitResult:
    """Loads the sweep and scores the engine (unless `engine` is false) and every candidate whose
    name matches the regex `only`."""
    sweep = load_sweep(fit.cases, fit.pattern, fit.where, fit.errors)
    cases = sorted({c.case for c in sweep.cells})
    candidates = [engine_candidate(cases)] if engine and cases else []
    wanted = re.compile(only) if only else None
    candidates += [c for c in fit.candidates() if wanted is None or wanted.search(c.name)]
    return FitResult(fit, sweep, [score(sweep.cells, c, keep) for c in candidates])

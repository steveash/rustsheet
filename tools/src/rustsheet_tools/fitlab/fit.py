"""A fit: one kernel's sweep, its candidates, and the run that scores and ranks them.

Every run splits the sweep into train and test cells (fitlab/holdout.py, decision 0188 R4.1) and
scores each candidate on both sides; the full-sweep score is their merge, so no cell is evaluated
twice.
"""

from __future__ import annotations

import re
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from rustsheet_tools.fitlab.holdout import (
    DEFAULT_MARGIN,
    HOLDOUT_PERMILLE,
    HOLDOUT_SALT,
    REFERENCE,
    Verdict,
    split,
    verdict,
)
from rustsheet_tools.fitlab.score import Candidate, ExcelError, Score, families, merged, rank, score
from rustsheet_tools.fitlab.sweep import Cell, Sweep, load_sweep
from rustsheet_tools.snapshot import show_cells

ENGINE = "engine (today)"


@dataclass
class Fit:
    """What to score: `cases` (`suite/name`), the formula `pattern` whose first group is the
    argument list, the candidates, an optional `where(cell)` filter (e.g. a sub-domain) and the
    `calls` cases whose Excel values stand in for nested calls (sweep.call_values). `margin` is
    the hold-out accept rule's allowed drop from the train to the test bit-exact rate, stated with
    the fit before any scoring (holdout.py)."""

    name: str
    description: str
    cases: list[str]
    pattern: str
    candidates: Callable[[], list[Candidate]]  # built lazily: some families are large
    where: Callable[[Cell], bool] | None = None
    errors: bool = False  # also score the cells whose Excel value is an error
    calls: list[str] = field(default_factory=list)  # cases giving Excel's value of nested calls
    notes: list[str] = field(default_factory=list)
    margin: float = DEFAULT_MARGIN


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
class Sides:
    """One candidate's hold-out scores and the accept verdict."""

    train: Score
    test: Score
    verdict: Verdict | None  # None for the correctly rounded candidate itself

    def to_json(self) -> dict[str, Any]:
        def counts(s: Score) -> dict[str, int]:
            return {"cells": s.total, "exact": s.exact, "within_4_ulp": s.ulp, "within_15_digits": s.digits15}

        return {
            "train": counts(self.train),
            "test": counts(self.test),
            "verdict": None if self.verdict is None else self.verdict.to_json(),
        }


@dataclass
class FitResult:
    fit: Fit
    sweep: Sweep
    scores: list[Score]  # over the whole sweep
    sides: dict[str, Sides] = field(default_factory=dict)  # by candidate name
    train_cells: int = 0
    test_cells: int = 0

    def holdout_json(self) -> dict[str, Any]:
        return {
            "salt": HOLDOUT_SALT,
            "hash": "sha256(salt NUL case NUL Sheet!A1), first 8 bytes big-endian, mod 1000",
            "test_permille": HOLDOUT_PERMILLE,
            "margin": self.fit.margin,
            "reference": REFERENCE,
            "train_cells": self.train_cells,
            "test_cells": self.test_cells,
        }

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
            "holdout": self.holdout_json(),
            "candidates": [
                {**s.to_json(worst), **(self.sides[s.name].to_json() if s.name in self.sides else {})}
                for s in rank(self.scores)
            ],
            "families": families(self.scores),
        }


def run(fit: Fit, *, engine: bool = True, only: str | None = None, keep: int = 10) -> FitResult:
    """Loads the sweep and scores the engine (unless `engine` is false) and every candidate whose
    name matches the regex `only`."""
    sweep = load_sweep(fit.cases, fit.pattern, fit.where, fit.errors, fit.calls)
    cases = sorted({c.case for c in sweep.cells})
    candidates = [engine_candidate(cases)] if engine and cases else []
    wanted = re.compile(only) if only else None
    candidates += [c for c in fit.candidates() if wanted is None or wanted.search(c.name)]
    train, test = split(sweep.cells)
    scored = [(c, score(train, c, keep), score(test, c, keep)) for c in candidates]
    reference = next((te for c, _, te in scored if c.name == REFERENCE), None)
    result = FitResult(fit, sweep, [], train_cells=len(train), test_cells=len(test))
    for c, tr, te in scored:
        result.scores.append(merged(tr, te, keep))
        judged = None if c.name == REFERENCE else verdict(tr, te, reference, fit.margin)
        result.sides[c.name] = Sides(tr, te, judged)
    return result

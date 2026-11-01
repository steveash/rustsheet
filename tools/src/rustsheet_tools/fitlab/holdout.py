"""The fit lab's mandatory hold-out split and accept rule (decision 0188 R4.1, T-3360).

Every fit splits its cells before scoring: a cell is held out (the *test* side) when the first
eight bytes of `sha256("rustsheet-fitlab-holdout/v1\\0<case>\\0<Sheet!A1>")`, read big-endian,
are below `HOLDOUT_PERMILLE` (300) modulo 1000. The split depends on nothing but the cell's own
case and address: the same cell is always on the same side, whatever else the sweep holds, the
order it is read in, or which candidates run. Candidates are scored on each side separately.

A candidate is **accepted** only when both hold:

- its test bit-exact rate is no more than the fit's `margin` (stated with the fit, before any
  scoring; default `DEFAULT_MARGIN`, 5 percentage points) below its train rate, and
- it beats the correctly rounded candidate on test: a higher test bit-exact count, the test
  within-4-ulp count breaking a tie (the fit lab's ranking).

A fit with no correctly rounded candidate, or a sweep with no test cell, accepts nothing.
"""

from __future__ import annotations

import hashlib
from collections.abc import Iterable
from dataclasses import dataclass, field
from fractions import Fraction
from typing import Any

from rustsheet_tools.fitlab.score import Score
from rustsheet_tools.fitlab.sweep import Cell

HOLDOUT_SALT = "rustsheet-fitlab-holdout/v1"
HOLDOUT_PERMILLE = 300  # about 30% held out (decision 0188 R4.1)
DEFAULT_MARGIN = 0.05  # the largest allowed drop from the train to the test bit-exact rate
REFERENCE = "correctly rounded"  # the candidate name every kernel module uses for it


def is_holdout(cell: Cell) -> bool:
    """Whether `cell` is on the test side of the split."""
    digest = hashlib.sha256(f"{HOLDOUT_SALT}\0{cell.case}\0{cell.key}".encode()).digest()
    return int.from_bytes(digest[:8], "big") % 1000 < HOLDOUT_PERMILLE


def split(cells: Iterable[Cell]) -> tuple[list[Cell], list[Cell]]:
    """(train, test), each in the order given."""
    train: list[Cell] = []
    test: list[Cell] = []
    for cell in cells:
        (test if is_holdout(cell) else train).append(cell)
    return train, test


def rate(s: Score) -> Fraction:
    """The bit-exact share of a side, exactly; 0 for a side with no cell."""
    return Fraction(s.exact, s.total) if s.total else Fraction(0)


@dataclass
class Verdict:
    """Whether a candidate is accepted, and every reason it is not."""

    accepted: bool
    reasons: list[str] = field(default_factory=list)

    def text(self) -> str:
        return "accept" if self.accepted else "refuse: " + "; ".join(self.reasons)

    def to_json(self) -> dict[str, Any]:
        return {"accepted": self.accepted, "reasons": self.reasons}


def verdict(train: Score, test: Score, reference: Score | None, margin: float) -> Verdict:
    """The accept rule for one candidate, given the correctly rounded candidate's test score."""
    reasons = []
    if not test.total:
        reasons.append("no hold-out cells")
    # Exact rationals: a drop of exactly the margin (55% to 50% at 5%) is within it.
    drop = rate(train) - rate(test)
    if drop > Fraction(str(margin)):
        reasons.append(
            f"test exact rate {float(rate(test)):.1%} is {float(drop):.1%} below train "
            f"{float(rate(train)):.1%} (margin {margin:.1%})"
        )
    if reference is None:
        reasons.append(f"no {REFERENCE!r} candidate to beat")
    elif (test.exact, test.ulp) <= (reference.exact, reference.ulp):
        reasons.append(
            f"does not beat {REFERENCE} on test ({test.exact} exact, {test.ulp} <=4 ulp against "
            f"{reference.exact}, {reference.ulp})"
        )
    return Verdict(not reasons, reasons)

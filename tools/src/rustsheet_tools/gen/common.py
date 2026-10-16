"""Pieces the probe policies share (`names-3d`, `operators-refs`, `tables`)."""

from __future__ import annotations

import random
from collections.abc import Callable, Sequence
from typing import ClassVar

REDRAWS = 200  # attempts to find a formula not yet in the case before accepting or dropping a repeat


class SeededDraws:
    """A policy's seeded random stream: every choice goes through `rng`, so a seed fixes the case.
    Subclasses set `numbers`, the literals `num` draws from."""

    numbers: ClassVar[Sequence[str]] = ()

    def __init__(self, rng: random.Random) -> None:
        self.rng = rng

    def pick(self, items: Sequence[str]) -> str:
        return self.rng.choice(items)

    def num(self) -> str:
        return self.pick(self.numbers)


def draw_unseen(draw: Callable[[], str], seen: set[str], redraws: int = REDRAWS) -> str | None:
    """`"=" + draw()` for a formula not yet in `seen` (and adds it), or None when `redraws` draws all
    repeat (it would be the same probe again)."""
    for _ in range(redraws):
        formula = "=" + draw()
        if formula not in seen:
            seen.add(formula)
            return formula
    return None

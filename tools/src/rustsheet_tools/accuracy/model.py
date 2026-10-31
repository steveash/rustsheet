"""What a Gate A definition is (SPEC-0029, "The generator and the sampler").

A `Definition` names a function's parameters in the catalog's order, each with its valid domain,
typical range and extreme range (each cited), and the mpmath function that computes its value at
the working precision. Definitions register themselves in `REGISTRY` from `defs/<family>.py`.
"""

from __future__ import annotations

import math
import random
from collections.abc import Callable, Sequence
from dataclasses import dataclass, field
from typing import Literal, TypeAlias

import mpmath

from rustsheet_tools.accuracy.literal import MAX_LITERAL, Arg

Kind = Literal["real", "int", "bool"]
MathArg: TypeAlias = "mpmath.mpf | bool"
"""An argument as the math sees it: the exact double (or integer) of the literal, or a logical."""


class DomainError(ValueError):
    """The point is outside the function's domain: Excel's error code is the target."""


class NoValueError(ValueError):
    """The point has no single math value to record (an overflow, or several roots with no
    evidenced root choice): the sampler redraws or drops it."""


@dataclass(frozen=True)
class Interval:
    """A real interval with open or closed ends; infinite ends are open."""

    lo: float = -math.inf
    hi: float = math.inf
    lo_open: bool = True
    hi_open: bool = True

    def contains(self, x: float) -> bool:
        """Whether `x` is inside."""
        above = x > self.lo or (x == self.lo and not self.lo_open)
        below = x < self.hi or (x == self.hi and not self.hi_open)
        return above and below

    def describe(self) -> str:
        """`[1, 1E+10]`, `(0, 1)`: the interval in the usual notation."""
        return f"{'(' if self.lo_open else '['}{self.lo:g}, {self.hi:g}{')' if self.hi_open else ']'}"


@dataclass(frozen=True)
class Param:
    """One parameter: its kind, valid domain, typical and extreme ranges and their source.

    `typical` and `extreme` are inclusive draw ranges inside the domain; a range spanning more
    than two decades is drawn log-uniform. `extreme` is a magnitude range: for a parameter whose
    domain has negative values the sign is drawn uniformly. `edges` are extra values worth a point
    beside the domain bounds (a documented limit such as df = 1E10 and its neighbour).
    """

    name: str
    kind: Kind
    domain: Interval = field(default_factory=Interval)
    typical: tuple[float, float] = (0.0, 1.0)
    extreme: tuple[float, float] | None = None
    edges: tuple[float, ...] = ()
    source: str = ""

    def finite_bounds(self) -> list[tuple[float, bool, bool]]:
        """The finite domain bounds as (value, open, is_lower)."""
        out: list[tuple[float, bool, bool]] = []
        if math.isfinite(self.domain.lo):
            out.append((self.domain.lo, self.domain.lo_open, True))
        if math.isfinite(self.domain.hi) and self.domain.hi < MAX_LITERAL:
            out.append((self.domain.hi, self.domain.hi_open, False))
        return out


TailMap = Callable[[mpmath.mpf, bool, Sequence[MathArg]], mpmath.mpf]
"""(tail probability q, upper tail?, the other arguments) → the x whose cdf is q (or 1 - q)."""
Draw = Callable[[str, random.Random], list[Arg]]
"""(stratum, rng) → one whole point, for a solver whose arguments are drawn together."""


@dataclass(frozen=True)
class Definition:
    """A function's Gate A definition. `value` raises `DomainError` outside the domain and
    `NoValueError` where no single value exists; `domain` adds rules beyond the parameters'
    intervals (returns why the point is invalid, or None). `tail` sets parameter `tail_param`
    from a tail probability (a distribution's x). `draw` replaces the per-parameter sampler for a
    solver (`solver`: the 300-point count). `sources` cite where the definition's math and
    ranges come from."""

    function: str
    params: tuple[Param, ...]
    value: Callable[[Sequence[MathArg]], mpmath.mpf]
    sources: tuple[str, ...]
    domain: Callable[[Sequence[MathArg]], str | None] | None = None
    tail: TailMap | None = None
    tail_param: str | None = None
    draw: Draw | None = None
    solver: bool = False

    def numeric_arity(self) -> int:
        """The number of real or integer parameters."""
        return sum(1 for p in self.params if p.kind != "bool")

    def check_domain(self, args: Sequence[MathArg]) -> None:
        """Raises `DomainError` when `args` are outside the parameters' intervals or `domain`."""
        for p, a in zip(self.params, args, strict=True):
            if p.kind != "bool" and not isinstance(a, bool) and not p.domain.contains(float(a)):
                msg = f"{p.name} outside {p.domain.describe()}"
                raise DomainError(msg)
        if self.domain is not None and (why := self.domain(args)) is not None:
            raise DomainError(why)

    def evaluate(self, args: Sequence[MathArg]) -> mpmath.mpf:
        """The math value at the current mpmath precision (domain checked first)."""
        self.check_domain(args)
        return self.value(args)


REGISTRY: dict[str, Definition] = {}


def register(definition: Definition) -> Definition:
    """Adds `definition` to `REGISTRY` (one per function name)."""
    if definition.function in REGISTRY:
        msg = f"duplicate Gate A definition for {definition.function}"
        raise ValueError(msg)
    REGISTRY[definition.function] = definition
    return definition


def solve_bracket(
    f: Callable[[mpmath.mpf], mpmath.mpf], a: mpmath.mpf, b: mpmath.mpf, max_steps: int = 6000
) -> mpmath.mpf:
    """The root of `f` in [a, b], where f(a) and f(b) have opposite signs (or one is 0), to the
    working precision: the Illinois variant of regula falsi, which keeps the bracket (so it
    cannot leave the domain) and converges superlinearly; a bisection step every fourth step
    bounds the worst case. NoValueError when the bracket has no sign change or does not shrink."""
    fa, fb = f(a), f(b)
    if fa == 0:
        return a
    if fb == 0:
        return b
    if (fa > 0) == (fb > 0):
        msg = "no sign change on the bracket"
        raise NoValueError(msg)
    eps = mpmath.mpf(2) ** (-mpmath.mp.prec + 2)
    tiny = mpmath.mpf(10) ** -320  # a root this close to 0 is 0 for Gate A's absolute floor
    side = 0
    for step in range(max_steps):
        if abs(b - a) <= eps * max(abs(a), abs(b), tiny):
            return a if abs(fa) < abs(fb) else b
        c = (a + b) / 2 if step % 4 == 3 else b - fb * (b - a) / (fb - fa)
        if not (min(a, b) < c < max(a, b)):
            c = (a + b) / 2
        fc = f(c)
        if fc == 0:
            return c
        if (fc > 0) == (fb > 0):
            b, fb = c, fc
            if side == -1:
                fa /= 2
            side = -1
        else:
            a, fa = c, fc
            if side == 1:
                fb /= 2
            side = 1
    msg = "root bracket did not converge"
    raise NoValueError(msg)

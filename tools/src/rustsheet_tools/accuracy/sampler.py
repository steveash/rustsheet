"""The stratified sampler of Gate A tables (SPEC-0029, "Strata"; decision 0188 R2).

A batch's points are split over five strata: typical 40%, tail 15% and domain edge 15% (together
the 30% of tails and edges), extreme-but-valid 20%, log-uniform 10%. Everything is drawn from one
`random.Random` seeded from `sha256(f"{FUNCTION}/{batch}")`: no clock, no environment, so the same
function, batch and tool version give the same points.
"""

from __future__ import annotations

import hashlib
import math
import random

import mpmath

from rustsheet_tools.accuracy.literal import MAX_LITERAL, MIN_LITERAL, Arg, neighbour, round15
from rustsheet_tools.accuracy.model import Definition, MathArg, Param

STRATA = ("typical", "tail", "edge", "extreme", "log-uniform")
SHARES = (0.40, 0.15, 0.15, 0.20, 0.10)
TAIL_DISTANCE = (1e-15, 1e-1)  # relative distance of a tail point from a finite domain bound
TAIL_PROBABILITY = (1e-300, 1e-1)  # a distribution's tail probability, drawn log-uniform
TAIL_MAP_DPS = 40  # enough for a 15-digit x


def seed(function: str, batch: str) -> int:
    """The batch's seed: `int(sha256(f"{FUNCTION}/{batch}"), 16) % 2**63`."""
    return int(hashlib.sha256(f"{function}/{batch}".encode()).hexdigest(), 16) % 2**63


def point_count(definition: Definition) -> int:
    """A first batch's size: 120 points up to two numeric parameters, 200 for three or more,
    300 for a solver."""
    if definition.solver:
        return 300
    return 120 if definition.numeric_arity() <= 2 else 200


def stratum_counts(count: int) -> dict[str, int]:
    """`count` split over the strata by `SHARES`, largest remainder first (ties in stratum order),
    so the counts sum to `count`: 120 → 48/18/18/24/12."""
    raw = [count * share for share in SHARES]
    counts = [math.floor(r) for r in raw]
    order = sorted(range(len(STRATA)), key=lambda i: (-(raw[i] - counts[i]), i))
    for i in order[: count - sum(counts)]:
        counts[i] += 1
    return dict(zip(STRATA, counts, strict=True))


def math_arg(param: Param, a: Arg) -> MathArg:
    """The math's view of a drawn argument: the exact double (or integer) or the logical."""
    if param.kind == "bool" or isinstance(a, bool):
        return bool(a)
    return mpmath.mpf(a)


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    """A draw log-uniform over [lo, hi], 0 < lo ≤ hi."""
    return math.exp(rng.uniform(math.log(lo), math.log(hi)))


def _spread(rng: random.Random, lo: float, hi: float) -> float:
    """Uniform over [lo, hi], or log-uniform when the range lies on one side of 0 and spans more
    than two decades."""
    if lo > 0 and hi / lo > 100:
        return log_uniform(rng, lo, hi)
    if hi < 0 and lo / hi > 100:
        return -log_uniform(rng, -hi, -lo)
    return rng.uniform(lo, hi)


class Sampler:
    """Draws the points of one batch for `definition` from `rng`."""

    def __init__(self, definition: Definition, rng: random.Random) -> None:
        self.definition = definition
        self.rng = rng
        self.edges = self._edge_candidates()

    # -- per-parameter draws --------------------------------------------------------------

    def finish(self, param: Param, x: float | bool) -> Arg:
        """A drawn value as a writable literal of the parameter's kind."""
        if param.kind == "bool":
            return bool(x)
        if param.kind == "int":
            return int(round15(float(round(float(x)))))
        return round15(float(x))

    def inside(self, param: Param, a: Arg) -> Arg:
        """`a`, or the nearest literal inside the domain when rounding left it."""
        if param.kind == "bool" or isinstance(a, bool) or param.domain.contains(float(a)):
            return a
        d = param.domain
        lower = float(a) < d.lo or (float(a) == d.lo and d.lo_open)
        bound = d.lo if lower else d.hi
        if param.kind == "int":
            step = 1 if lower else -1
            start = math.floor(bound) if lower else math.ceil(bound)
            value = start + step if (bound == start and (d.lo_open if lower else d.hi_open)) else start
            if not d.contains(float(value)):
                value += step
            return int(value)
        return neighbour(bound, up=lower)

    def typical(self, param: Param) -> Arg:
        """A draw over the parameter's typical range."""
        if param.kind == "bool":
            return self.rng.random() < 0.5
        lo, hi = param.typical
        return self.inside(param, self.finish(param, _spread(self.rng, lo, hi)))

    def extreme(self, param: Param) -> Arg:
        """A draw over the parameter's extreme magnitude range, signed when the domain allows."""
        if param.extreme is None:
            return self.typical(param)
        x = _spread(self.rng, *param.extreme)
        if param.domain.lo < 0 and self.rng.random() < 0.5:
            x = -x
        return self.inside(param, self.finish(param, x))

    def whole_domain(self, param: Param) -> Arg:
        """A draw log-uniform over the parameter's whole valid domain (sign uniform if signed)."""
        if param.kind == "bool":
            return self.rng.random() < 0.5
        floor = 1.0 if param.kind == "int" else MIN_LITERAL
        lo = max(param.domain.lo, -MAX_LITERAL)
        hi = min(param.domain.hi, MAX_LITERAL)
        negative = hi <= 0 or (lo < 0 and self.rng.random() < 0.5)
        top = -lo if negative else hi
        bottom = max(floor, -hi if negative else lo, floor)
        x = top if top <= bottom else log_uniform(self.rng, bottom, top)
        return self.inside(param, self.finish(param, -x if negative else x))

    # -- strata --------------------------------------------------------------------------

    def draw(self, stratum: str, index: int) -> list[Arg]:
        """Point number `index` of `stratum`."""
        d = self.definition
        if d.draw is not None:
            drawn = d.draw(stratum, self.rng)
            return [self.finish(p, a) for p, a in zip(d.params, drawn, strict=True)]
        if stratum == "tail":
            return self._tail(index)
        if stratum == "edge":
            return self._edge(index)
        if stratum == "extreme":
            return self._extreme()
        if stratum == "log-uniform":
            return [self.whole_domain(p) for p in d.params]
        return [self.typical(p) for p in d.params]

    def _extreme(self) -> list[Arg]:
        params = self.definition.params
        candidates = [i for i, p in enumerate(params) if p.extreme is not None]
        chosen = {i for i in candidates if self.rng.random() < 0.5}
        if candidates and not chosen:
            chosen = {self.rng.choice(candidates)}
        return [self.extreme(p) if i in chosen else self.typical(p) for i, p in enumerate(params)]

    def _tail_targets(self) -> list[int | None]:
        """Round-robin targets of tail points: None for the tail map, else a parameter index."""
        d = self.definition
        targets: list[int | None] = [None] if d.tail is not None else []
        targets += [i for i, p in enumerate(d.params) if p.kind == "real" and p.finite_bounds()]
        return targets

    def _tail(self, index: int) -> list[Arg]:
        params = self.definition.params
        args = [self.typical(p) for p in params]
        targets = self._tail_targets()
        if not targets:
            return args
        target = targets[index % len(targets)]
        if target is None:
            return self._tail_mapped(args)
        param = params[target]
        bounds = param.finite_bounds()
        bound, _, lower = bounds[(index // len(targets)) % len(bounds)]
        distance = log_uniform(self.rng, *TAIL_DISTANCE) * max(abs(bound), 1.0)
        args[target] = self.inside(param, self.finish(param, bound + distance if lower else bound - distance))
        return args

    def _tail_mapped(self, args: list[Arg]) -> list[Arg]:
        """Sets the tail parameter (a distribution's x) to the quantile of a log-uniform tail
        probability, in the lower or upper tail, so a cdf point is really in the tail."""
        d = self.definition
        assert d.tail is not None and d.tail_param is not None
        target = next(i for i, p in enumerate(d.params) if p.name == d.tail_param)
        q = mpmath.mpf(log_uniform(self.rng, *TAIL_PROBABILITY))
        upper = self.rng.random() < 0.5
        others = [math_arg(p, a) for p, a in zip(d.params, args, strict=True)]
        try:
            with mpmath.workdps(TAIL_MAP_DPS):
                x = d.tail(q, upper, others)
        except (ValueError, ZeroDivisionError, OverflowError):
            return args  # the typical draw stands (e.g. the other arguments are invalid)
        param = d.params[target]
        if not mpmath.isfinite(x) or abs(x) > MAX_LITERAL:
            return args
        args[target] = self.inside(param, self.finish(param, float(x)))
        return args

    def _edge_candidates(self) -> list[tuple[int, Arg]]:
        """Every (parameter index, value) edge: each finite bound itself, its nearest literal
        inside, the nearest invalid literal outside, and the parameter's extra `edges`."""
        out: list[tuple[int, Arg]] = []
        for i, p in enumerate(self.definition.params):
            if p.kind == "bool":
                continue
            values: list[float] = []
            for bound, is_open, lower in p.finite_bounds():
                if p.kind == "int":
                    step = 1.0 if lower else -1.0
                    values += [bound, bound + step, bound - step]
                else:
                    values += [bound, neighbour(bound, up=lower)]
                    if not is_open:
                        values.append(neighbour(bound, up=not lower))
            values += list(p.edges)
            for v in values:
                candidate = self.finish(p, v)
                if (i, candidate) not in out:
                    out.append((i, candidate))
        return out

    def _edge(self, index: int) -> list[Arg]:
        args = [self.typical(p) for p in self.definition.params]
        if self.edges:
            target, value = self.edges[index % len(self.edges)]
            args[target] = value
        return args

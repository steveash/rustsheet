"""T-3509: the gamma family's held-out probe, a blind log-uniform sweep (decision 0188 R4.2).

Writes the six eval/suites/probes-numeric/cases/t3509-blind-*.case.json files (4,100 cells): every
argument is drawn log-uniform over the function's whole domain, independently of any fit, and
written as a literal of at most 15 significant digits. See docs/research/t3509/README.md for what
the probe is for, the rules that keep it a hold-out, and how to refresh it.

    python3 -I docs/research/t3509/sweep.py                 # rewrite the committed draw (SEED)
    python3 -I docs/research/t3509/sweep.py --seed 20270115 # refresh: a new seed, new points

A refresh seed must not be in USED_SEEDS (the seeds whose points have been seen, and so may have
been fitted to); add it there in the refresh's PR, and set SEED to it.

Deterministic: one `random.Random(seed)` draws every case in the order below, so the same seed
writes the same bytes. Seed 20261031 is the 2026-10-31 sweep at main 3003a898b (/tmp/rsx/gen.py of
that session), in its draw order, so its cells are that sweep's points.
"""

from __future__ import annotations

import argparse
import json
import math
import random
from collections.abc import Callable
from pathlib import Path

FIRST_SEED = 20261031  # the 2026-10-31 sweep (its GAMMA.INV quirk is kept so its cells stay its points)
SEED = FIRST_SEED  # the draw committed now
USED_SEEDS = (FIRST_SEED,)  # every seed whose points were ever filled: never draw from these again
CASES = Path(__file__).resolve().parents[3] / "eval" / "suites" / "probes-numeric" / "cases"
PREFIX = "t3509-blind-"
N = 500  # points per distribution
N_GAMMA = 300  # points per GAMMALN and GAMMA


def log_uniform(rng: random.Random, lo: float, hi: float) -> float:
    """10^U(log10 lo, log10 hi)."""
    return float(10 ** rng.uniform(math.log10(lo), math.log10(hi)))


def lit(v: float) -> str:
    """A formula literal of at most 15 significant digits."""
    return f"{v:.15G}"


Rows = list[dict[str, str]]  # one dict of column letter -> formula per row


def gamma_dist(rng: random.Random, _first_draw: bool) -> Rows:
    """GAMMA.DIST cdf (A) and pdf (B) at scale 1: shape 1E-3 to 1E7, x/shape 1E-12 to 1E3."""
    rows: Rows = []
    for _ in range(N):
        a = log_uniform(rng, 1e-3, 1e7)
        x = a * log_uniform(rng, 1e-12, 1e3)
        args = f"{lit(x)},{lit(a)},1"
        rows.append({"A": f"=GAMMA.DIST({args},TRUE)", "B": f"=GAMMA.DIST({args},FALSE)"})
    return rows


def chisq_dist_rt(rng: random.Random, _first_draw: bool) -> Rows:
    """CHISQ.DIST.RT: integer df 1 to 2000, x/df 1E-8 to 60."""
    rows: Rows = []
    for _ in range(N):
        df = max(1, round(log_uniform(rng, 1, 2000)))
        x = df * log_uniform(rng, 1e-8, 60)
        rows.append({"A": f"=CHISQ.DIST.RT({lit(x)},{df})"})
    return rows


def gamma_inv(rng: random.Random, first_draw: bool) -> Rows:
    """GAMMA.INV at scale 1: shape 1E-2 to 1E7, tail t 1E-300 to 0.5 on either side (p = t or 1 - t)."""
    rows: Rows = []
    for _ in range(N):
        a = log_uniform(rng, 1e-2, 1e7)
        t = log_uniform(rng, 1e-300, 0.5)
        p = t if rng.random() < 0.5 else 1 - t
        if p >= 1:
            # 1 - t is 1 for t below 1.1E-16. The first draw wrote p = 0.5 there, so 227 of its
            # 500 cells are the median; later draws redraw t where 1 - t is below 1.
            p = 0.5 if first_draw else 1 - log_uniform(rng, 1.2e-16, 0.5)
        rows.append({"A": f"=GAMMA.INV({lit(p)},{lit(a)},1)"})
    return rows


def chisq_inv_rt(rng: random.Random, _first_draw: bool) -> Rows:
    """CHISQ.INV.RT: p 1E-300 to 0.999, integer df 1 to 2000."""
    rows: Rows = []
    for _ in range(N):
        df = max(1, round(log_uniform(rng, 1, 2000)))
        p = log_uniform(rng, 1e-300, 0.999)
        rows.append({"A": f"=CHISQ.INV.RT({lit(p)},{df})"})
    return rows


def poisson_dist(rng: random.Random, _first_draw: bool) -> Rows:
    """POISSON.DIST cdf (A) and pmf (B): k 0 to 1E5, mean/(k + 1) 1E-3 to 30."""
    rows: Rows = []
    for _ in range(N):
        k = round(log_uniform(rng, 1, 1e5)) - 1
        mean = (k + 1) * log_uniform(rng, 1e-3, 30)
        rows.append(
            {"A": f"=POISSON.DIST({k},{lit(mean)},TRUE)", "B": f"=POISSON.DIST({k},{lit(mean)},FALSE)"}
        )
    return rows


def gammaln_gamma(rng: random.Random, _first_draw: bool) -> Rows:
    """GAMMALN (A) at x 1E-6 to 1E7, log-uniform; GAMMA (B) at x uniform in -30 to 171.6, off the poles."""
    rows: Rows = []
    for _ in range(N_GAMMA):
        x = log_uniform(rng, 1e-6, 1e7)
        g = rng.uniform(-30, 171.6)
        if abs(g - round(g)) < 1e-6 and g <= 0:
            g += 0.5
        rows.append({"A": f"=GAMMALN({lit(x)})", "B": f"=GAMMA({lit(g)})"})
    return rows


# (case suffix, what its columns hold, the drawing function(rng, first draw?)), in draw order.
SECTIONS: list[tuple[str, str, Callable[[random.Random, bool], Rows]]] = [
    ("gamma-dist", "GAMMA.DIST(x,a,1,TRUE) in A and FALSE in B", gamma_dist),
    ("chisq-dist-rt", "CHISQ.DIST.RT(x,df) in A", chisq_dist_rt),
    ("gamma-inv", "GAMMA.INV(p,a,1) in A", gamma_inv),
    ("chisq-inv-rt", "CHISQ.INV.RT(p,df) in A", chisq_inv_rt),
    ("poisson-dist", "POISSON.DIST(k,mean,TRUE) in A and FALSE in B", poisson_dist),
    ("gammaln-gamma", "GAMMALN(x) in A and GAMMA(x) in B", gammaln_gamma),
]


def write(seed: int) -> int:
    """Writes every case for `seed`; returns the number of cells."""
    rng = random.Random(seed)
    total = 0
    for suffix, columns, draw in SECTIONS:
        cells = {
            f"{col}{row}": text
            for row, entry in enumerate(draw(rng, seed == FIRST_SEED), start=1)
            for col, text in entry.items()
        }
        total += len(cells)
        doc = (draw.__doc__ or "").strip()
        case = {
            "format": "rustsheet-case/v1",
            "description": (
                f"T-3509 held-out probe (never fit to it; docs/research/t3509/README.md): {columns}. {doc} "
                f"Seed {seed}, docs/research/t3509/sweep.py; scored per region against Excel and 50-digit math by "
                "docs/research/t3509/score.py (decision 0188 R2, R4.2)."
            ),
            "tags": ["M9", "probe", "numeric", "statistics", "sweep", "holdout", "T-3509"],
            "sheets": [{"name": "S", "cells": cells}],
            "expected": {},
        }
        (CASES / f"{PREFIX}{suffix}.case.json").write_text(
            json.dumps(case, indent=1) + "\n", encoding="utf-8"
        )
    return total


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--seed", type=int, default=SEED, help=f"default {SEED} (the committed draw)")
    seed = parser.parse_args().seed
    if seed != SEED and seed in USED_SEEDS:
        raise SystemExit(f"seed {seed} was already filled: its points are no longer held out (USED_SEEDS)")
    print(f"seed {seed}: {write(seed)} cells under {CASES}/{PREFIX}*.case.json")


if __name__ == "__main__":
    main()

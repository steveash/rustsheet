"""F-distribution sweep statistics: the T-2220 numbers behind statistics.md "Dense F sweep".

Usage (needs mpmath, which the repo does not depend on; e.g. in a throwaway venv):

    python3 -m venv /tmp/mpv && /tmp/mpv/bin/pip install mpmath
    /tmp/mpv/bin/python docs/research/t2220_f_dist_sweep_stats.py

Reads the five `probes-numeric/t2220-f-dist-sweep-*` cases and their oracle sidecars and compares each
cell with a 50-digit reference (regularized incomplete beta; density from the closed form) rounded
to a double: correctly rounded share per function and per df pair, ulp histogram, relative error,
and how often F.DIST(TRUE) + F.DIST.RT is exactly 1. The grid itself was generated once (df pairs
x {0.1..100}; 30 log-spaced x over 1E-6..1E6 at d1 = 1 or d2 = 1), one formula per cell.
"""

from __future__ import annotations

import json
import struct
from collections import Counter, defaultdict
from pathlib import Path

import mpmath as mp

mp.mp.dps = 50
CASES = Path(__file__).resolve().parents[2] / "eval/suites/probes-numeric/cases"
NAMES = ["cdf", "pdf", "rt", "d1-1", "d2-1"]


def load(name: str) -> dict[str, dict]:
    stem = CASES / f"t2220-f-dist-sweep-{name}"
    case = json.loads(stem.with_suffix(".case.json").read_text())
    oracle = json.loads(stem.with_suffix(".oracle.json").read_text())["sheets"]["Calc"]
    return {
        case["hypotheses"]["Calc!" + c]: oracle[c] for c in case["sheets"][0]["cells"]
    }


def parse(label: str) -> tuple[str, float, int, int]:
    kind, x, d1, d2 = label.split("/")
    return kind, float(x[2:]), int(d1[3:]), int(d2[3:])


def exact(kind: str, x: float, d1: int, d2: int) -> mp.mpf:
    mx, a, b = mp.mpf(str(x)), mp.mpf(d1), mp.mpf(d2)
    z = a * mx / (a * mx + b)
    if kind == "cdf":
        return mp.betainc(a / 2, b / 2, 0, z, regularized=True)
    if kind == "rt":
        return mp.betainc(a / 2, b / 2, z, 1, regularized=True)
    return (
        a ** (a / 2)
        * b ** (b / 2)
        * mx ** (a / 2 - 1)
        / (a * mx + b) ** ((a + b) / 2)
        / mp.beta(a / 2, b / 2)
    )


def ulps(got: float, ref: float) -> int:
    bits = lambda v: struct.unpack("<q", struct.pack("<d", v))[0]
    return abs(bits(got) - bits(ref))


def main() -> None:
    cells: dict[str, dict] = {}
    for name in NAMES:
        cells.update(load(name))
    print("non-numeric cells:", [h for h, v in cells.items() if v["t"] != "n"])
    exact_by_kind: dict[str, list[int]] = defaultdict(lambda: [0, 0])
    exact_by_df: dict[tuple, list[int]] = defaultdict(lambda: [0, 0])
    hist: dict[str, Counter] = defaultdict(Counter)
    far_tail = []
    for label, v in cells.items():
        kind, x, d1, d2 = parse(label)
        ref = float(exact(kind, x, d1, d2))
        got = v["v"]
        for tally in (exact_by_kind[kind], exact_by_df[(kind, d1, d2)]):
            tally[0] += 1
            tally[1] += got == ref
        if ref and abs(got - ref) / abs(ref) > 1e-9:
            far_tail.append((kind, x, d1, d2, got, ref))
        else:
            hist[kind][min(ulps(got, ref), 10)] += 1
    print("correctly rounded (cells, exact):", dict(exact_by_kind))
    for kind in ("cdf", "pdf", "rt"):
        full = [k[1:] for k, t in exact_by_df.items() if k[0] == kind and t[0] == t[1]]
        print(kind, "df pairs with every cell correctly rounded:", full)
        print(kind, "ulp histogram (10 = 10 or more):", sorted(hist[kind].items()))
    print("cells more than 1E-9 relative from the reference:", far_tail)
    sums = Counter()
    for label, v in cells.items():
        kind, x, d1, d2 = parse(label)
        if kind == "cdf":
            rt = cells[f"rt/x={label.split('/')[1][2:]}/d1={d1}/d2={d2}"]
            sums[v["v"] + rt["v"] == 1.0] += 1
    print("cdf + rt == 1.0:", dict(sums))


if __name__ == "__main__":
    main()

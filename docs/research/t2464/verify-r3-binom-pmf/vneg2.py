"""NEGBINOM.DIST(f,r,p,FALSE): the claimed shifted kernel p * dbinom_raw(r-1, f+r-1) against R's dnbinom
(r/(f+r)) * dbinom_raw(r, f+r), each with r3-binom-pmf's frozen kernels (k = 0 / k = n closed forms included),
plus the ulp spread of Excel's NEG against p times Excel's own BINOM.DIST(0, f, p) where the probe pairs them.
Usage: vneg2.py"""

from __future__ import annotations

import collections
import math
from fractions import Fraction

import numpy as np

import vdata as D
import vlib as V
import vmodels as M

LDB = np.longdouble


def kern(name: str, k: int, n: int, p: float) -> float | None:
    return M.worker_model(name, dict(kern="BIN", k=k, n=n, p=Fraction(p)))


def forms(name: str, f: int, r: int, p: float) -> dict:
    out = {}
    b = kern(name, r - 1, f + r - 1, p)
    out["shift fl(p*B(r-1,f+r-1))"] = None if b is None else float(LDB(p) * LDB(b))
    b2 = kern(name, r, f + r, p)
    if b2 is not None:
        out["R fl(fl(r/(f+r))*B(r,f+r))"] = float(LDB(float(LDB(r) / LDB(f + r))) * LDB(b2))
        out["R fl(fl(r*B)/(f+r))"] = float(LDB(float(LDB(r) * LDB(b2))) / LDB(f + r))
    return out


def main() -> None:
    for name in ("old", "hold", "v"):
        rows = [r for r in D.load(name) if r["kern"] == "NEG" and r["k"] >= 1 and r["n"] >= 1]
        for mdl in ("L-lcx/LF1", "L-lcxx/LF0"):
            g = collections.defaultdict(lambda: collections.defaultdict(list))
            for r in rows:
                reg = "r=1" if r["n"] == 1 else "r>1"
                for c, v in forms(mdl, r["k"], r["n"], float(r["p"])).items():
                    g[reg][c].append((v, r["excel"]))
            for reg in sorted(g):
                for c, pairs in g[reg].items():
                    print(f"{name:5s} {mdl:11s} {reg} {c:28s} {V.fmt(V.score(pairs))}")
    # pairs in the fresh probe: B row j = NEGBINOM(f,1,p) and A row j = BINOM(0,f,p), same (f, p)
    rows = D.load("v")
    a = {(r["n"], r["p"]): r["excel"] for r in rows if r["kern"] == "BIN" and r["k"] == 0}
    hist = collections.Counter()
    for r in rows:
        if r["kern"] == "NEG" and r["n"] == 1 and (r["k"], r["p"]) in a:
            bx = a[(r["k"], r["p"])]
            if bx == 0 or r["excel"] == 0:
                continue
            prod = Fraction(float(r["p"])) * Fraction(bx)
            hist[round(float((Fraction(r["excel"]) - prod) / Fraction(math.ulp(r["excel"]))), 1)] += 1
    print("v: Excel NEG(f,1,p) - p*Excel BINOM(0,f,p) (exact product), in ulps of NEG:", sorted(hist.items()))


if __name__ == "__main__":
    main()

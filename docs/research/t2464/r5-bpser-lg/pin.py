"""T-3570 r5-bpser-lg: pin the lnGamma(a0) double inside bpser's L, per a0 of probe t3570-r5-bpser-lg.

Model (r2-bpser-small section 2, 1 < b0 < 8): I = fl(EXP(fl64(alx - L)) * s), alx = fl(a0 LN x), s = w3's forward
series, L = fl(g + fl(GL(b0) - GL(fl(a0 + b0)))), with GL(b0) and GL(a0 + b0) Excel's own (columns K and J). Only g is
free. For each a0, scan the doubles g within +-R ulps of Excel's GAMMALN(a0) (column I) and keep those that make all
8 x cells exact: the pinned set. Then report where the candidate forms fall:
  gl      Excel's GAMMALN(a0) (column I)
  g1ln53  fl(gamln1(a0) - LN(a0))  (TOMS gamln for a0 < 1, double code with Excel's LN)
  acc     lnGamma(a0) correctly rounded
Hold-out: md5 odd half of a0. --control plants data from the model with g = g1ln53 and with g = gl (positive control).
usage: pin.py [--control] [--list]"""
from __future__ import annotations

import hashlib
import json
import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-bpser-small"))
import lib  # noqa: E402
from lib import LN, mpf, sx  # noqa: E402

NAME = "t3570-r5-bpser-lg"
R = 6
K = lib.Model(lib.bx.BEST, lib.V())


def hold(a0: float) -> bool:
    return int(hashlib.md5(repr(a0).encode()).hexdigest(), 16) % 2 == 1


def load():
    out = json.loads((lib.CASES / f"{NAME}.oracle.json").read_text())["sheets"]["Calc"]
    meta = json.load(open(HERE / f"{NAME}.meta.json"))
    gb = {}
    for row, a0, b0, xs in meta:
        k = out.get(f"K{row}")
        if k:
            gb[b0] = float(k["v"])

    def num(c):
        v = out.get(c)
        return float(v["v"]) if v and v.get("t") == "n" else None

    pts = []
    for row, a0, b0, xs in meta:
        cells = [(x, num(f"{col}{row}")) for col, x in zip("ABCDEFGH", xs)]
        pts.append(dict(a0=a0, b0=b0, xs=xs, cells=cells, gl=num(f"I{row}"), glab=num(f"J{row}"), glb=gb[b0]))
    return pts


def predict(p, g, x):
    a0, b0 = p["a0"], p["b0"]
    L = sx("+", g, sx("-", p["glb"], p["glab"]))
    alx = mpf(sx("*", a0, LN(x)))
    return sx("*", lib.EXPX(lib.fl64(alx - mpf(L))), lib.Model.series_w3(a0, b0, x))


def cands(p):
    a0 = p["a0"]
    with mpmath.workdps(40):
        acc = float(mpmath.loggamma(mpf(a0)))
    return {"gl": p["gl"], "g1ln53": sx("-", K.gamln1(a0), LN(a0)), "acc": acc}


def pinned(p, cells):
    base = p["gl"]
    g = base
    for _ in range(R):
        g = math.nextafter(g, -math.inf)
    out = []
    for k in range(-R, R + 1):
        if all(predict(p, g, x) == e for x, e in cells):
            out.append(k)
        g = math.nextafter(g, math.inf)
    return out  # offsets in ulps from Excel's GAMMALN(a0)


def off(v, base):
    return round((v - base) / math.ulp(base))


def main() -> None:
    pts = load()
    control = "--control" in sys.argv
    tally = defaultdict(Counter)
    for i, p in enumerate(pts):
        cs = cands(p)
        cells = p["cells"]
        if control:  # plant: alternate the true form by row parity
            truth = "g1ln53" if i % 2 else "gl"
            cells = [(x, predict(p, cs[truth], x)) for x, _ in cells]
        pin = pinned(p, cells)
        key = (f"{p['a0']:.2f}", "hold" if hold(p["a0"]) else "tune")
        t = tally[key]
        t["n"] += 1
        t["pinned"] += bool(pin)
        t["unique"] += len(pin) == 1
        offs = {nm: off(v, p["gl"]) for nm, v in cs.items()}
        for nm, o in offs.items():
            t[nm] += o in pin
        if offs["gl"] != offs["g1ln53"]:
            t["differ"] += 1
            for nm in ("gl", "g1ln53"):
                t[f"{nm}|d"] += offs[nm] in pin
        for nm, v in cs.items():
            t[f"{nm}:cells"] += sum(predict(p, v, x) == e for x, e in cells)
        t["cells"] += len(cells)
        if control:
            t["truth"] += off(cs[truth], p["gl"]) in pin
        if "--list" in sys.argv:
            truth = f" truth={truth}" if control else ""
            print(f"a0={p['a0']!r} b0={p['b0']} pin={pin} cands={offs}{truth}")
    cols = ["n", "pinned", "unique", "gl", "g1ln53", "acc", "differ", "gl|d", "g1ln53|d",
            "cells", "gl:cells", "g1ln53:cells", "acc:cells", "truth"]
    print("center split " + " ".join(f"{c:>8s}" for c in cols))
    for key in sorted(tally):
        print(f"{key[0]:6s} {key[1]:5s} " + " ".join(f"{tally[key][c]:8d}" for c in cols))
    tot = Counter()
    for t in tally.values():
        tot.update(t)
    print("all          " + " ".join(f"{tot[c]:8d}" for c in cols))


if __name__ == "__main__":
    main()

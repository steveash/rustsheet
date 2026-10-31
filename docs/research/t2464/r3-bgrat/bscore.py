"""T-2512 r3-bgrat: score bgrat models on the recorded calls (calls.pkl, exact.pkl; optional qxl.pkl from the probe).

Models (G = bgrat's returned w + increment; the lower tail is G or fl(0.5 - G + 0.5) as the branch dictates):
  M0     r2-tdist FINAL as is (TOMS bgrat in doubles with Excel's pieces)
  EXQ    RN(w + F): the exact I at ln x = lnx, i.e. only lnx is Excel's
  XLQ    RN(w + F * Qxl / Q): r2-tdist's b = 1/2 identity with Excel's measured incomplete gamma Qxl(b, z) (probe)
usage: score.py [qsrc]   qsrc names the qxl.pkl column used as Qxl (default: all)"""
from __future__ import annotations

import math
import pickle
import sys
from collections import defaultdict
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-bratio"))
import score_lib as sl  # noqa: E402

from exact import zdouble  # noqa: E402


def lower_of(res_lower: float, g_model: float):
    """How the branch turns G into the lower tail: identity or the complement fl(0.5 - G + 0.5)."""
    if res_lower == g_model:
        return lambda g: g
    return lambda g: 0.5 - g + 0.5


def cells(half: str = "all"):
    d = pickle.load(open(HERE / "calls.pkl", "rb"))
    ex = pickle.load(open(HERE / "exact.pkl", "rb"))
    out = []
    for i, c in d["calls"].items():
        a, b, x, y, w, lnx, g = c
        r = d["rows"][i]
        if (a, b, lnx) not in ex:
            continue
        if half != "all" and r["hold"] != (half == "hold"):
            continue
        br = d["res"][i][2]
        out.append(dict(row=r, a=a, b=b, x=x, y=y, w=w, lnx=lnx, g=g, br=br, fn=lower_of(d["res"][i][0], g),
                        F=ex[(a, b, lnx)][0], Q=ex[(a, b, lnx)][1], z=zdouble(a, b, lnx)))
    return out


def model(c, q=None):
    with mpmath.workdps(40):
        inc = c["F"] if q is None else c["F"] * mpmath.mpf(q) / c["Q"]
        return c["fn"](float(mpmath.mpf(c["w"]) + inc))


def report(cs, models):
    groups = defaultdict(list)
    for c in cs:
        groups[(c["br"].replace("/sw", ""), "b=1/2" if c["b"] == 0.5 else "b!=1/2")].append(c)
    names = list(models)
    print("| branch | b | n | " + " | ".join(names) + " |")
    print("|---|---|---|" + "---|" * len(names))
    for k in sorted(groups) + [("ALL", "")]:
        g = cs if k[0] == "ALL" else groups[k]
        line = f"| {k[0]} | {k[1]} | {len(g)} |"
        for m in names:
            vals = [(models[m](c), c["row"]["excel"]) for c in g]
            vals = [(v, e) for v, e in vals if v is not None]
            n, e, f, w = sl.tally(vals)
            line += f" {e}/{f}/{w} ({n}) |"
        print(line)


if __name__ == "__main__":
    cs = cells(sys.argv[1] if len(sys.argv) > 1 else "all")
    qx = pickle.load(open(HERE / "qxl.pkl", "rb")) if (HERE / "qxl.pkl").exists() else {}
    models = {"M0": lambda c: c["fn"](c["g"]), "EXQ": lambda c: model(c)}
    for src in sorted({s for (s, _b, _z) in qx}):
        models[f"XLQ:{src}"] = (lambda s: lambda c: model(c, qx[(s, c["b"], c["z"])]) if (s, c["b"], c["z"]) in qx
                                else None)(src)
    report(cs, models)

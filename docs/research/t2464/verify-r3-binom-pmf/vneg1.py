"""NEGBINOM.DIST(f,1,p,FALSE) = p q^f: the cleanest test of the claimed NEG = fl(p * dbinom_raw(r-1, f+r-1, p, q)),
because dbinom_raw(0, f, p, q) is the k = 0 closed form EXP(fl(f ln1m(p))) (bit-exact at k = 0).
Compares how p is combined: stored product (claim), extended EXP times p, exponent shifts by LN p.
Usage: vneg1.py   (all sets: old, n, hold, v if filled)"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np

import vdata as D
import vlib as V
import vmodels as M

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r3-loader-bd0"))
import r3lib as R3  # noqa: E402  (EXP V6 on an extended input, extended output)

LDB = np.longdouble


def forms(f: int, p: float) -> dict:
    E0 = M.edge_exponent(0, f, p)  # fl(f * ln1m(p)), the k = 0 closed form's stored exponent
    lp = M.xl("LN", [p])[0]
    ex = R3.exp_v6(LDB(E0), ext=True)
    return {
        "claim fl(p*EXP(E0))": float(LDB(p) * LDB(float(ex))),
        "p*EXP(E0) ext EXP": float(LDB(p) * ex),
        "EXP(fl(E0+LN p))": R3.exp_v6(LDB(float(LDB(E0) + LDB(lp)))),
        "EXP(E0+LN p ext)": R3.exp_v6(LDB(E0) + LDB(lp)),
        "EXP(fl((f*ln1m)+LNp) ext prod)": R3.exp_v6(LDB(f) * LDB(M.ln1m(p)) + LDB(lp)),
        "fl(p*fl(EXP(E0)))/x87 same": float(LDB(p) * LDB(M.xl("EXP", [E0])[0])),
    }


def main() -> None:
    for name in ("old", "n", "hold", "v"):
        rows = [r for r in D.load(name) if r["kern"] == "NEG" and r["n"] == 1 and r["k"] >= 1]
        if not rows:
            continue
        cols: dict = {}
        for r in rows:
            for c, v in forms(r["k"], float(r["p"])).items():
                cols.setdefault(c, []).append(v)
        print(f"== {name}: NEG r=1, {len(rows)} rows, distinct {len({D.key(r) for r in rows})}")
        print(f"   {'CR':32s} {V.fmt(V.score((V.cr(r), r['excel']) for r in rows))}")
        for c, vs in cols.items():
            print(f"   {c:32s} {V.fmt(V.score(zip(vs, (r['excel'] for r in rows))))}")


if __name__ == "__main__":
    main()

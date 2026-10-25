"""Per run: least-squares slope of D (units) on the local rounding errors of s1 = fl(1 + n/mx) and s2 = fl(1 + mx/n)
(units, relative), for Excel and for brcomp/x models (a positive control: a model's own D shows its own slopes).
round 3 (fact 3): Excel's slope is about -h on the s of the smaller ratio when that ratio is > 0.375."""
import sys

import mpmath
import numpy as np

import fblib as B

mpf = mpmath.mpf
U = mpf(2) ** 53
SET = sys.argv[1] if len(sys.argv) > 1 else "a"
MODELS = {"w5": {"gamln": "toms"}, "rs": {"gamln": "toms", "args": "rs"}, "rsm": {"gamln": "toms", "args": "rsm"}}


def deltas(x, m, n):
    mx = m * x
    r1, r2 = n / mx, mx / n
    s1, s2 = 1.0 + r1, 1.0 + r2
    e1 = float((mpf(s1) - (1 + mpf(r1))) / (1 + mpf(r1)) * U)
    e2 = float((mpf(s2) - (1 + mpf(r2))) / (1 + mpf(r2)) * U)
    return e1, e2


def slopes(D, E):
    A = np.column_stack([np.ones(len(D)), np.array(E)])
    c, *_ = np.linalg.lstsq(A, np.array(D), rcond=None)
    return c[1], c[2]


rows = B.load(SET)
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)
print(f"# {SET}: slopes on (s1, s2) in units of h; r = smaller ratio at the run start")
for blk, rr in runs.items():
    r0 = rr[0]
    m, n = r0["d1"], r0["d2"]
    h = (m + n) / 2
    E = [deltas(r["x"], m, n) for r in rr]
    rat = min(m * r0["x"] / n, n / (m * r0["x"]))
    De = [float((mpmath.log(r["excel"]) - mpmath.log(r["exact"])) * U) for r in rr]
    out = [f"{blk:22s} r={rat:.3f} a0={min(m, n) / 2:5.1f}", "xl %+5.2f %+5.2f" % tuple(v / h for v in slopes(De, E))]
    for k, o in MODELS.items():
        Dm = [float((mpmath.log(B.model(r["x"], m, n, o)) - mpmath.log(r["exact"])) * U) for r in rr]
        out.append(f"{k} " + "%+5.2f %+5.2f" % tuple(v / h for v in slopes(Dm, E)))
    print(" | ".join(out))

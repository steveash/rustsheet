"""Does a single value of Excel's internal lnGamma(0.5) (exact + d, d in units of 2^-60) explain the x-runs
(0.5, 6.25) and (1.5, 0.5) under z64 (and z53)? L = fl64(lg(a) + fl64(lg(b) - lg(a+b)))."""
import sys
from lib import *
from series import accum2
M = mpmath.mpf
R = load(cum=True, branch="direct")
LG05 = mpmath.loggamma(M(0.5))
def run(blk, ds):
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
    E = np.array([r["excel"] for r in rows])
    s1, _ = accum2(x, a, b, mode="x87", rec="R4", tf="T0", start="1/a")
    A, B = a[0], b[0]
    alx = [M(v) for v in mul(a, xl("LN", x))]
    def lg(v, d):
        if v == 0.5: return fl64(LG05 + M(d) * M(2) ** -60)
        if v < 1: return fl64(mpmath.loggamma(M(v)))
        return M(xl("GAMMALN", [v])[0])
    out = {}
    for d in ds:
        L = fl64(lg(A, d) + fl64(lg(B, d) - lg(float(add(A, B)), d)))
        z = [p - L for p in alx]
        c64 = int((mul(np.array([fl53(mpmath.exp(fl64(v))) for v in z]), s1) == E).sum())
        c53 = int((mul(xl("EXP", np.array([fl53(v) for v in z])), s1) == E).sum())
        out[d] = (c64, c53)
    return out
ds = list(range(int(sys.argv[1]), int(sys.argv[2]), int(sys.argv[3])))
r1, r2 = run("xruns-1:3", ds), run("xruns-1:6", ds)
for d in ds:
    print(f"  d={d:+4d} (x2^-60)  (0.5,6.25) z64 {r1[d][0]:4d} z53 {r1[d][1]:4d}   (1.5,0.5) z64 {r2[d][0]:4d} z53 {r2[d][1]:4d}")

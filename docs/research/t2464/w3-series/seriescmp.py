"""Series-variant comparison under the best prefactor per block (z53 with free k in 1/4-ulp(z) steps, and z64 with
free fine offset): forward accumulation from 1/a (Cephes-style terms c_n/(a+n)), from 1 (a*c_n/(a+n)), TOMS bpser
(sum then a*sum+1), Cephes pseries order (sum n>=2, + t1, + 1/a), exact S. Counts are best over the offset grid."""
import sys
from lib import *
from model import exponent
from series import accum2, bpser_sum, cephes_pseries_sum
M = mpmath.mpf
R = load(cum=True, branch="direct")
blocks = sys.argv[1:] or ["xruns-1:3", "xruns-1:6", "xruns-2:6", "absweep:2", "absweep:6"]
for blk in blocks:
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = np.array([r["a"] for r in rows]); b = np.array([r["b"] for r in rows])
    E = np.array([r["excel"] for r in rows])
    z = exponent(x, a, b, big=8.0)
    I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
    Sx = [i / (M(xi) ** M(ai) / M(ai) / mpmath.beta(M(ai), M(bi))) for i, xi, ai, bi in zip(I, x, a, b)]
    V = {}
    for mode in ("x87", "dbl", "ext"):
        V[f"fwd from 1/a, {mode}"] = ("s", accum2(x, a, b, mode=mode, rec="R4", tf="T0", start="1/a")[0])
        V[f"fwd from 1, {mode}"] = ("S", accum2(x, a, b, mode=mode, rec="R4", tf="T1", start=1.0)[0])
        V[f"bpser a*sum+1, {mode}"] = ("S", bpser_sum(x, a, b, mode=mode, tol_kind="unchanged")[0])
        V[f"cephes order, {mode}"] = ("s", cephes_pseries_sum(x, a, b, mode=mode))
    V["exact S (dbl)"] = ("S", np.array([float(s) for s in Sx]))
    zs = [M(math.ulp(float(v))) for v in z]
    P53 = {}; P64 = {}
    for k in range(-8, 9):
        P53[k] = xl("EXP", np.array([fl53(v + M(k) / 4 * u) for v, u in zip(z, zs)]))
    for k in range(-24, 25):
        P64[k] = np.array([fl53(mpmath.exp(fl64(v + M(k) / 16 * u))) for v, u in zip(z, zs)])
    print(f"== {blk} n={len(E)} a={a[0]:g} b={b[0]:g}")
    out = []
    for nm, (kind, sv) in V.items():
        def fin(P):
            return mul(P, sv) if kind == "s" else mul(div(P, a), sv)
        c53 = max((int((fin(P) == E).sum()), k) for k, P in P53.items())
        c64 = max((int((fin(P) == E).sum()), k) for k, P in P64.items())
        out.append((max(c53[0], c64[0]), nm, c53, c64))
    for o in sorted(out, reverse=True):
        print(f"   {o[1]:24s} z53 {o[2][0]:5d} (k={o[2][1]:+d}/4)  z64 {o[3][0]:5d} (k={o[3][1]:+d}/16)")
save_exact()

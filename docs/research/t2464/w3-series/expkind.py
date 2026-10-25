"""Which exp? Excel's EXP (V6 model) vs correctly rounded exp vs libm (glibc) exp vs x87 extended exp (no rounding
to double), in I = EXP(fl(fl(a*LN x) - L))/a * S, on a fixed-(a,b) block with free L."""
import sys
from lib import *
from series import bpser_sum
M = mpmath.mpf
blk = sys.argv[1]; K = int(sys.argv[2]) if len(sys.argv) > 2 else 20
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows]); aa = np.full_like(x, A); bb = np.full_like(x, B)
I = [exact_cdf(xi, A, B) for xi in x]
T = [M(xi) ** M(A) / (M(A) * mpmath.beta(M(A), M(B))) for xi in x]
Sx = [i / t for i, t in zip(I, T)]
Sb, _ = bpser_sum(x, aa, bb, mode="ext", tol_kind="unchanged")
lnB0 = float(mpmath.log(mpmath.beta(M(A), M(B))))
for lnname in ("xlLN", "crLN"):
    lnx = xl("LN", x) if lnname == "xlLN" else np.array([float(mpmath.log(M(v))) for v in x])
    alx = mul(A, lnx)
    best = {}
    for k in range(-K, K + 1):
        L = lnB0 + k * math.ulp(lnB0)
        z = sub(alx, L)
        exps = {"xlEXP": xl("EXP", z), "crEXP": np.array([float(mpmath.exp(M(v))) for v in z]),
                "libm": np.exp(z)}
        ext = [mpmath.exp(M(v)) for v in z]  # unrounded exp, then extended ops
        for en, e in exps.items():
            P = div(e, A)
            c = int((mul(P, Sb) == E).sum())
            c2 = int((np.array([float(M(p) * s) for p, s in zip(P, Sx)]) == E).sum())
            best[en] = max(best.get(en, (0,)), (c, c2, k))
        # extended exp: P = fl64(exp(z)) / a in ext, times S, one final rounding
        c = int((np.array([fl53(fl64(fl64(ev) / A) * M(sb)) for ev, sb in zip(ext, Sb)]) == E).sum())
        c2 = int((np.array([fl53(ev / A * s) for ev, s in zip(ext, Sx)]) == E).sum())
        best["exact exp, one rounding"] = max(best.get("exact exp, one rounding", (0,)), (c, c2, k))
    for en, v in best.items():
        print(f"{blk} {lnname} {en:26s} bpser-ext {v[0]:5d} S-exact {v[1]:5d}  L=ex{v[2]:+d}")
print("Excel EXP == crEXP fraction on z:", np.mean(xl("EXP", z) == np.array([float(mpmath.exp(M(v))) for v in z])))
print("Excel LN == crLN:", np.mean(xl("LN", x) == np.array([float(mpmath.log(M(v))) for v in x])))

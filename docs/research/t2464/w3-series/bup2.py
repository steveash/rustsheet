"""L140 (bup + bpser) Excel-styled, with x recomputed from y (x' = 1 - fl(1-x)) in chosen places:
 tail bpser(a, b', X1), bup prefactor ln x from X2; bup powers of y. Tie response and exact counts on the
a,b > 1 x-runs (direct side, x <= 0.7)."""
import itertools
from lib import *
from model import lg_mixed, model
from dispatch import branch
M = mpmath.mpf
R = load(cum=True, branch="direct")
def tie_c(xs, vals, ex):
    d = np.array([float((M(float(sub(1.0, v))) - (1 - M(v))) / (1 - M(v))) / U for v in xs])
    e = rel_units(vals, ex)
    up, dn = e[d > 1e-9], e[d < -1e-9]
    return (up.mean() - dn.mean()) / (2 * np.abs(d[np.abs(d) > 1e-9]).mean())
def bup_sum(y, a, bpr, n):
    w = np.ones_like(y); d = np.ones_like(y)
    apb = add(a, bpr); ap1 = add(bpr, 1.0)
    for i in range(int(n.max()) - 1):
        act = i < n - 1
        dn = mul(d, mul(div(add(apb, float(i)), add(ap1, float(i))), y))
        d = np.where(act, dn, d); w = np.where(act, add(w, dn), w)
    return w
for blk in ("xruns-1:0", "xruns-1:1", "xruns-1:2", "xruns-1:5", "xruns-2:3"):
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
    E = np.array([r["excel"] for r in rows]); y = sub(1.0, x); xp = sub(1.0, y)
    ex = [exact_cdf(v, A, B) for v in x]
    n = int(B); bp = B - n
    if bp == 0: n -= 1; bp = 1.0
    a0 = np.full_like(x, A); bpa = np.full_like(x, bp); nn = np.full(len(x), n)
    la, lb, lab = lg_mixed(bpa), lg_mixed(a0), lg_mixed(add(a0, bpa))
    Lb = [fl64(la[i] + fl64(lb[i] - lab[i])) for i in range(len(x))]
    w = bup_sum(y, a0, bpa, nn)
    out = []
    for X1n, X2n, lyn in itertools.product(("x", "x'"), ("x", "x'"), ("log1p(-x)", "LN(y)", "log1p(-x')")):
        X1 = x if X1n == "x" else xp
        X2 = x if X2n == "x" else xp
        tail = model(X1, a0, bpa, big=8.0, zmode="z64")
        lny = {"log1p(-x)": np.array([float(mpmath.log1p(-M(v))) for v in x]), "LN(y)": xl("LN", y),
               "log1p(-x')": np.array([float(mpmath.log1p(-M(v))) for v in xp])}[lyn]
        zz = [M(p) + M(q) - L for p, q, L in zip(mul(A, xl("LN", X2)), mul(bp, lny), Lb)]
        P = div(np.array([fl53(mpmath.exp(fl64(v))) for v in zz]), bp)
        I = add(mul(P, w), tail)
        out.append((int((I == E).sum()), tie_c(x, I, ex), X1n, X2n, lyn))
    print(f"== {blk} a={A} b={B}: Excel c {tie_c(x, E, ex):+.2f}")
    for o in sorted(out, reverse=True)[:5]:
        print("   exact %4d  c %+.2f  tail x=%s, bup ln x from %s, ln y=%s" % o)

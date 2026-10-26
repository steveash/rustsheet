"""Hypergeometric form: I = x^a y^b/(a B) * H, H = 2F1(a+b,1;a+1;x) = sum (a+b)_n/(a+1)_n x^n.
Prefactor exponent z' = fl(fl(a*LN x) + fl(b*log1p(-x))) - L in several groupings; regression and exact counts."""
import sys, itertools, collections
from lib import *
M = mpmath.mpf
blk = sys.argv[1]; K = int(sys.argv[2]) if len(sys.argv) > 2 else 20
rows = sorted([r for r in load(cum=True, branch="direct") if r["block"] == blk], key=lambda r: r["x"])
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows]); mA, mB = M(A), M(B)
I = [exact_cdf(xi, A, B) for xi in x]; Bt = mpmath.beta(mA, mB)
X = [M(v) for v in x]
Hx = [i / (v ** mA * (1 - v) ** mB / (mA * Bt)) for i, v in zip(I, X)]
Hd = np.array([float(h) for h in Hx])
err = rel_units(E, I)
lnB0 = float(mpmath.log(Bt))
lnx = xl("LN", x); alx = mul(A, lnx)
l1p = np.array([float(mpmath.log1p(-v)) for v in X])          # correctly rounded log1p(-x)
lny_ln = xl("LN", sub(1.0, x))
eln = np.array([float((M(l) - mpmath.log(v))) * mA / U for l, v in zip(lnx, X)])
res = collections.defaultdict(list)
for lyn, ly in (("log1p", l1p), ("LN(1-x)", lny_ln)):
    bly = mul(B, ly)
    ely = np.array([float(M(l) - mpmath.log1p(-v)) * mB / U for l, v in zip(ly, X)])
    for k in range(-K, K + 1):
        L = lnB0 + k * math.ulp(lnB0)
        for gn, z in (("(alx+bly)-L", sub(add(alx, bly), L)), ("(alx-L)+bly", add(sub(alx, L), bly)),
                      ("alx+(bly-L)", add(alx, sub(bly, L)))):
            ez = xl("EXP", z)
            P = div(ez, A)
            for hn, m in (("P*fl(H)", mul(P, Hd)), ("P*H", np.array([float(M(p) * h) for p, h in zip(P, Hx)]))):
                res[(lyn, gn, hn)].append((int((m == E).sum()), k))
        if k == 0:
            ez = xl("EXP", sub(add(alx, bly), L))
            eexp = np.array([float(M(e) / mpmath.exp(M(zz)) - 1) / U for e, zz in zip(ez, sub(add(alx, bly), L))])
            Xm = np.column_stack([eln, ely, eexp, np.ones_like(eln)]).astype(float)
            c, *_ = np.linalg.lstsq(Xm, err, rcond=None)
            print(f"{lyn}: regression ln {c[0]:+.2f} lny {c[1]:+.2f} exp(z') {c[2]:+.2f} const {c[3]:+.2f} resid {(err - Xm@c).std():.3f}")
for kk, v in sorted(res.items(), key=lambda t: -max(t[1])[0])[:12]:
    c, k = max(v)
    print(f"  {c:5d} L=ex{k:+d}  {kk}")
save_exact()

"""Per block (fixed a, b): S variants x prefactor forms x free lnB (+-K ulp): bit-exact counts of Excel's direct cdf."""
import sys, collections
from lib import *
from series import bpser_sum, cephes_pseries_sum
M = mpmath.mpf
blk = sys.argv[1]; K = int(sys.argv[2]) if len(sys.argv) > 2 else 30
rows = [r for r in load(cum=True, branch="direct") if r["block"] == blk]
x = np.array([r["x"] for r in rows]); A = rows[0]["a"]; B = rows[0]["b"]
E = np.array([r["excel"] for r in rows])
T = [M(xi) ** M(A) / (M(A) * mpmath.beta(M(A), M(B))) for xi in x]
Sx = [exact_cdf(xi, A, B) / t for xi, t in zip(x, T)]
Sd = np.array([float(v) for v in Sx])
aa = np.full_like(x, A); bb = np.full_like(x, B)
Ss = {"S exact->dbl": Sd}
for mode in ("dbl", "x87", "ext"):
    for tk in ("eps_a", "unchanged"):
        for var in ("toms", "simple", "nb"):
            S, sm = bpser_sum(x, aa, bb, mode=mode, tol_kind=tk, eps=1e-15 if tk == "eps_a" else 0, variant=var)
            Ss[f"bpser {var} {mode} {tk}"] = S
            Ss[f"bpser {var} {mode} {tk} [sum]"] = ("sum", sm)
    Ss[f"cephes {mode}"] = ("ceph", cephes_pseries_sum(x, aa, bb, mode=mode))
lnx = xl("LN", x); alx = mul(A, lnx)
lnBex = float(mpmath.log(mpmath.beta(M(A), M(B))))
cands = [lnBex]
for d in (math.inf, -math.inf):
    v = lnBex
    for k in range(K):
        v = math.nextafter(v, d); cands.append(v)
cands.sort()
xl("EXP", np.concatenate([sub(alx, L) for L in cands] + [np.array([-L for L in cands])]))
expalx = xl("EXP", alx)
res = collections.defaultdict(list)
for L in cands:
    e1 = xl("EXP", sub(alx, L)); eL = xl("EXP", np.array([-L]))[0]
    Pa = div(e1, A)          # EXP(alx-L)/a
    P0 = e1                  # EXP(alx-L)
    Pb = div(mul(expalx, eL), A)
    for sn, S in Ss.items():
        if isinstance(S, tuple):
            kind, v = S
            if kind == "sum":   # ans = P; ans *= (a*sum + 1) vs ans = P0 * (1/a + sum)?
                m = mul(P0, add(div(1.0, A), v)); res[(sn, "EXP(alx-L)*(1/a+sum)")].append((int((m == E).sum()), L))
                m = mul(Pa, add(mul(A, v), 1.0)); res[(sn, "EXP(alx-L)/a*(a*sum+1)")].append((int((m == E).sum()), L))
            else:  # cephes s = 1/a + ... ; I = s * x^a / B
                m = mul(P0, v); res[(sn, "EXP(alx-L)*s")].append((int((m == E).sum()), L))
                m = mul(v, mul(expalx, eL)); res[(sn, "s*(EXP(alx)*EXP(-L))")].append((int((m == E).sum()), L))
                m = mul(mul(v, eL), expalx); res[(sn, "(s*EXP(-L))*EXP(alx)")].append((int((m == E).sum()), L))
            continue
        for pn, P in (("EXP(alx-L)/a", Pa), ("EXP(alx)*EXP(-L)/a", Pb)):
            m = mul(P, S); res[(sn, pn)].append((int((m == E).sum()), L))
        m = div(mul(P0, S), A); res[(sn, "EXP(alx-L)*S/a")].append((int((m == E).sum()), L))
out = []
for k, lst in res.items():
    lst.sort(reverse=True)
    out.append((lst[0][0], k, (lst[0][1] - lnBex) / math.ulp(lnBex)))
out.sort(reverse=True)
print(blk, A, B, "n", len(E))
for c, k, d in out[:25]:
    print(f"  {c:5d}  lnB=ex{d:+.0f}  {k[0]:38s} {k[1]}")
save_exact()

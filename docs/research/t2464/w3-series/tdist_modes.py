"""T.DIST.RT 0<t<=1, n>=17 (and 3..16): series summation modes/stopping rules/assemblies for the inner I_z(0.5,n/2)."""
import sys
exec(open(__file__.replace("tdist_modes.py", "tdist_holdout.py")).read().split("cands = {")[0])
from series import cephes_pseries_sum
L = np.where(b <= 8, Lgl, Lex)
ez = xl("EXP", sub(alx, L)); P = div(ez, 0.5)
masks = {"n>=17": n >= 17, "3..16": (n >= 3) & (n <= 16)}
res = []
for mode in ("dbl", "x87", "ext"):
    for stop in ("unchanged", "eps_a", "rel"):
        for var in ("toms", "simple", "nb"):
            S, sm = bpser_sum(z, a, b, mode=mode, tol_kind=stop, eps=1e-15 if stop == "eps_a" else 2.0 ** -53, variant=var, nmax=20000)
            for asm, H in (("P*(1+a*sum)", mul(P, S)), ("EXP(z)*(1/a+sum)", mul(ez, add(2.0, sm))),
                           ("P + P*(a*sum)", add(P, mul(P, mul(0.5, sm))))):
                T = tail(H)
                res.append((f"bpser {var} {mode} {stop} {asm}", {k: score(T[m], E[m])[0] for k, m in masks.items()}))
    sC = cephes_pseries_sum(z, a, b, mode=mode, nmax=20000)
    res.append((f"cephes {mode} EXP(z)*s", {k: score(tail(mul(ez, sC))[m], E[m])[0] for k, m in masks.items()}))
res.sort(key=lambda r: -r[1]["n>=17"])
for nm, d in res[:12] + res[-4:]:
    print(f"  {nm:44s} {d}")

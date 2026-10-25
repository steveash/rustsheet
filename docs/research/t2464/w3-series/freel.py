"""Series S variants with the lnB constant FREE per (a,b) group (fitted on the tune half, scored on the hold-out half).
This decouples the (separately studied) Beta-function constant from the x-dependent part: prefactor
P = EXP(fl(fl(a*LN x) - L))/a, I = P * S.  Also an all-extended variant and the correctly rounded value scaled by
a free constant (positive/negative control for what a free constant alone buys).
Usage: python freel.py [region] [K]   region in a<1, b<1, ab>1-bx<=0.7, ab>1-bx>0.7, all
"""

import collections
import sys

from lib import *
from series import bpser_sum, cephes_pseries_sum

M = mpmath.mpf
REG = sys.argv[1] if len(sys.argv) > 1 else "a<1"
K = int(sys.argv[2]) if len(sys.argv) > 2 else 40


def region(r):
    a, b, x = r["a"], r["b"], r["x"]
    if min(a, b) < 1:
        return "a<1" if a < 1 else "b<1"
    return "ab>1-bx<=0.7" if b * x <= 0.7 else "ab>1-bx>0.7"


rows = [r for r in load(cum=True, branch="direct") if r["a"] != 1.0 and r["b"] != 1.0]
if REG != "all":
    rows = [r for r in rows if region(r) == REG]
tune, hold = split_half(rows)
tid = {id(r) for r in tune}
groups = collections.defaultdict(list)
for r in rows:
    groups[(r["a"], r["b"]) if r["block"] == "be" or "xruns" in r["block"] else r["block"]].append(r)

tot = collections.defaultdict(lambda: np.zeros(6, dtype=int))  # tune ex, hold ex, hold w4, hold d15, n_t, n_h
for gk, rs in groups.items():
    x = np.array([r["x"] for r in rs])
    a = np.array([r["a"] for r in rs])
    b = np.array([r["b"] for r in rs])
    E = np.array([r["excel"] for r in rs])
    it = np.array([id(r) in tid for r in rs])
    I = [exact_cdf(xi, ai, bi) for xi, ai, bi in zip(x, a, b)]
    lnB = [mpmath.log(mpmath.beta(M(ai), M(bi))) for ai, bi in zip(a, b)]
    T = [M(xi) ** M(ai) / M(ai) / mpmath.exp(l) for xi, ai, l in zip(x, a, lnB)]
    Sx = [i / t for i, t in zip(I, T)]
    Svars = {"S exact": Sx, "S exact->dbl": np.array([float(s) for s in Sx])}
    for mode in ("dbl", "x87", "ext"):
        for stop in ("unchanged", "eps_a"):
            Svars[f"bpser {mode} {stop}"] = bpser_sum(
                x, a, b, mode=mode, tol_kind=stop, eps=1e-15, nmax=20000
            )[0]
        Svars[f"cephes {mode}"] = ("c", cephes_pseries_sum(x, a, b, mode=mode, nmax=20000))
    alx = mul(a, xl("LN", x))
    lnB0 = np.array([float(l) for l in lnB])
    # candidate L: lnB0 + k ulp (per cell lnB0 differs only in sweep groups; then k is a common offset)
    ulp0 = np.array([math.ulp(v) for v in lnB0])
    Ls = [lnB0 + k * ulp0 for k in range(-K, K + 1)]
    xl("EXP", np.concatenate([sub(alx, L) for L in Ls]))
    res = collections.defaultdict(list)
    for k, L in zip(range(-K, K + 1), Ls):
        ez = xl("EXP", sub(alx, L))
        P = div(ez, a)
        for sn, S in Svars.items():
            if isinstance(S, tuple):
                m = mul(ez, S[1])
            elif sn == "S exact":
                m = np.array([float(M(p) * s) for p, s in zip(P, S)])
            else:
                m = mul(P, S)
            res[sn].append((k, m))
        # all-extended: fl53(exp(fl64(alx - L))/a * S_exact)
        if len(rs) <= 1100:
            m = np.array(
                [fl53(mpmath.exp(fl64(M(p) - M(l))) / M(ai) * s) for p, l, ai, s in zip(alx, L, a, Sx)]
            )
            res["all-ext (exp, S exact)"].append((k, m))
        # control: correctly rounded value scaled by (1 + k*2^-54)
        res["control CR*(1+k 2^-54)"].append(
            (k, np.array([float(i * (1 + M(k) * M(2.0) ** -54)) for i in I]))
        )
    for sn, lst in res.items():
        best = max(lst, key=lambda t: int((t[1][it] == E[it]).sum()))
        m = best[1]
        sh = score(m[~it], E[~it])
        tot[sn] += np.array([int((m[it] == E[it]).sum()), sh[0], sh[1], sh[2], it.sum(), (~it).sum()])
    cr = np.array([float(v) for v in I])
    sh = score(cr[~it], E[~it])
    tot["correctly rounded (no fit)"] += np.array(
        [int((cr[it] == E[it]).sum()), sh[0], sh[1], sh[2], it.sum(), (~it).sum()]
    )
save_exact()
print(
    f"region {REG}: {len(rows)} cells in {len(groups)} groups; free L per group fitted on tune; hold-out scores"
)
print(f"{'variant':32s} {'tune exact':>10s} {'hold exact/w4/d15':>22s}  (n tune / hold)")
for sn, v in sorted(tot.items(), key=lambda t: -t[1][1]):
    print(f"{sn:32s} {v[0]:10d} {v[1]:8d}/{v[2]:6d}/{v[3]:6d}   ({v[4]}/{v[5]})")

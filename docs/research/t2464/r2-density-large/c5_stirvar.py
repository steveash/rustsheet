"""stirlerr variants on the sweep fingerprint: candidate C = LN(fl(a+b)-1) + Cs_variant - LF0/2 (Excel LN pieces,
x87 stored; the c4 evidence), need as in c4 (exact a, b constant).  Variants for stirlerr(v), v not a table value:
  ex  : exact;  tab: w6 (table / GL formula / series);  g1: GL(v+1) - (v+.5)LN(v) + v - c (x87 stored)
  gE  : same kept extended, stored once;  gG: LN(GAMMA(v+1)) in place of GL;  g0: GL(v) + LN(v) in place of GL(v+1)
Prints rho and rms(need - cand) per sweep."""
import sys
import numpy as np
import mpmath
import dlib as L
import loader as LD
import model as M

U = mpmath.mpf(2) ** 53
rows = L.load()
SQ = mpmath.mpf(M.H.M_LN_SQRT_2PI)
D, R64 = L.D, L.R64
mp = mpmath.mpf
LNm = lambda v: mp(L.LN(float(v)))  # noqa: E731


def st_var(v, var):
    v = mp(v)
    if var == "ex":
        return mpmath.loggamma(v + 1) - (v + 0.5) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))
    if var == "tab":
        return M.H.Arith("D").stirlerr(float(v))
    if var == "gG":
        lg = LNm(L.excelmath.excel1("GAMMA", float(D(v + 1))))
    elif var == "g0":
        lg = mp(D(mp(L.GL(float(v))) + LNm(v)))
    else:
        lg = mp(L.GL(float(D(v + 1))))
    if var == "gE":
        return mp(D(R64(R64(R64(lg - R64(R64(v + 0.5) * LNm(v))) + v) - SQ)))
    t = mp(D(mp(D(v + 0.5)) * LNm(v)))
    return mp(D(mp(D(mp(D(lg - t)) + v)) - SQ))


VARS = ["ex", "tab", "g1", "gE", "gG", "g0"]
for blk in sys.argv[1:]:
    idx = L.blocks(rows)[blk]
    need, cand = [], {v: [] for v in VARS}
    for i in idx:
        r = rows[i]
        a, b = r["a"], r["b"]
        A, B = mp(a), mp(b)
        k, n = A - 1, A + B - 2
        nk = n - k
        Cex = -mpmath.log(mpmath.beta(A, B)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
        X = LD.xpart(r["x"], a, b, **M.BEST_X)[0]
        need.append(L.err_units(r["excel"], r["exact"]) - L.err_units(L.exp_v6(mp(D(Cex + X))), r["exact"]))
        ks, ns, nks, Cs, lf0, L5 = M.shape_consts(a, b, {})
        for var in VARS:
            Csv = st_var(ns, var) - st_var(ks, var) - st_var(nks, var)
            cand[var].append(float((L5 + Csv - lf0 / 2 - Cex) * U))
    need = np.array(need)
    print(f"{blk}: need mean {need.mean():+.2f} sd {need.std():.2f}")
    for var in VARS:
        c = np.array(cand[var])
        rho = np.corrcoef(c, need)[0, 1] if c.std() > 1e-9 else float("nan")
        print(f"    {var:4s} mean {c.mean():+8.2f} sd {c.std():6.2f} rho {rho:+.2f} rms {np.sqrt(((need - c) ** 2).mean()):6.2f}", flush=True)

"""Shape-constant fingerprint on the shape sweeps (w2 absweep: x fixed, a or b stepped by single ulps).
Per cell: need = Excel err - model err with the exact constant (E = fl(C_exact + X)), units 2^-53; compared with each
candidate constant's own error (rms of need - cand, and correlation).  Candidates as in c2_const.py plus
  GLB  : -(GL(a) + GL(b) - GL(a+b)) + k ln(k/n) + nk ln(nk/n)       (exact logs)
  GLBx : same with k*LN(k/n), nk*LN(nk/n) stored and the sum in x87 stored order"""
import sys
import numpy as np
import mpmath
import dlib as L
import loader as LD
import model as M
import c2_const as C2  # noqa: F401  (stir_g)

TB = C2.TB
U = mpmath.mpf(2) ** 53
rows = L.load()
for blk in sys.argv[1:]:
    idx = L.blocks(rows)[blk]
    L.gl_prefetch([v for i in idx for v in (rows[i]["a"], rows[i]["b"], rows[i]["a"] + rows[i]["b"],
                                            rows[i]["a"] + rows[i]["b"] - 1, rows[i]["a"] - 1 + 1)])
    need, cand = [], {}
    for i in idx:
        r = rows[i]
        a, b = r["a"], r["b"]
        A, B = mpmath.mpf(a), mpmath.mpf(b)
        ks, ns, nks, Cs, lf0, L5 = M.shape_consts(a, b, {})
        k, n, nk = mpmath.mpf(ks), mpmath.mpf(ns), mpmath.mpf(nks)
        Cex = -mpmath.log(mpmath.beta(A, B)) + k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
        X = LD.xpart(r["x"], a, b, **M.BEST_X)[0]
        need.append(L.err_units(r["excel"], r["exact"]) - L.err_units(L.exp_v6(mpmath.mpf(L.D(Cex + X))), r["exact"]))
        Csg = C2.stir_g(ns) - C2.stir_g(ks) - C2.stir_g(nks)
        ste = lambda v: mpmath.loggamma(mpmath.mpf(v) + 1) - (mpmath.mpf(v) + 0.5) * mpmath.log(v) + v - mpmath.log(mpmath.sqrt(2 * mpmath.pi))  # noqa: E731,E501
        Cse = ste(ns) - ste(ks) - ste(nks)
        klog = k * mpmath.log(k / n) + nk * mpmath.log(nk / n)
        glb = -(mpmath.mpf(L.GL(a)) + L.GL(b) - L.GL(a + b))
        t1 = mpmath.mpf(L.D(k * mpmath.mpf(L.LN(float(L.D(k / n))))))
        t2 = mpmath.mpf(L.D(nk * mpmath.mpf(L.LN(float(L.D(nk / n))))))
        glbx = mpmath.mpf(L.D(mpmath.mpf(L.D(mpmath.mpf(L.D(mpmath.mpf(L.D(mpmath.mpf(L.GL(a + b)) - L.GL(a))) - L.GL(b))) + t1)) + t2))
        cs = {"LDt": L5 + Cs - lf0 / 2, "LDg": L5 + Csg - lf0 / 2, "LDe": L5 + Cse - lf0 / 2,
              "T1": -mpmath.mpf(TB.betaln(a, b)) + klog, "GLB": glb + klog, "GLBx": glbx}
        for nm, c in cs.items():
            cand.setdefault(nm, []).append(float((c - Cex) * U))
    need = np.array(need)
    r0 = rows[idx[0]]
    print(f"{blk} ({r0['a']:.17g},{r0['b']:.17g}) need mean {need.mean():+.2f} sd {need.std():.2f}")
    for nm, c in cand.items():
        c = np.array(c)
        rho = np.corrcoef(c, need)[0, 1] if c.std() > 1e-9 else float("nan")
        print(f"    {nm:5s} cand mean {c.mean():+8.2f} sd {c.std():6.2f}  rho {rho:+.2f}  rms(need-cand) {np.sqrt(((need - c) ** 2).mean()):.2f}")

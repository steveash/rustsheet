"""Positive controls on synthetic data at the real points.
1. Free-constant scan (s2_scan.run): synthetic Excel = EXP_V6(fl(C0 + 3.3 units + X_best)); the scan must find
   ~+3.3 units and score ~100% on the hold-out; a wrong structure (exact Loader, cut 0.1) must score lower.
2. y-tie response estimator (d5 method): on synthetic Loader-x87 data it must give (b-1)-(a+b-2)y; on synthetic
   log-form data (b-1)*LN(fl(1-x)) it must give b-1."""
import numpy as np
import mpmath
import dlib as L
import loader as LD
import model as M
import s2_scan as S

U = mpmath.mpf(2) ** 53
rows = S.rows
for blk in ("xruns-1:0", "xruns-2:3"):
    idx = L.blocks(rows)[blk]
    r0 = rows[idx[0]]
    X0 = LD.xpart(r0["x"], r0["a"], r0["b"], ar="m", lnmode="m", ymode="ex")[0]
    C = mpmath.log(r0["exact"]) - X0 + mpmath.mpf(3.3) / U
    saved = {i: rows[i]["excel"] for i in idx}
    for i in idx:
        rows[i]["excel"] = L.exp_v6(mpmath.mpf(L.D(C + LD.xpart(rows[i]["x"], rows[i]["a"], rows[i]["b"], **M.BEST_X)[0])))
    print("control 1, truth = best knobs, C0+3.3u:")
    S.run(blk, "ar=x,lnmode=xl,addar=e,cut=0.2,sar=e,xar=e,comb=x")
    S.run(blk, "ar=m,lnmode=m,cut=0.1,comb=x")
    for i in idx:
        rows[i]["excel"] = saved[i]

print("control 2 (y-tie response):")
for blk in ("xruns-1:0", "xruns-1:2", "xruns-2:3"):
    idx = L.blocks(rows)[blk]
    r0 = rows[idx[0]]
    a, b = r0["a"], r0["b"]
    lnB = mpmath.log(mpmath.beta(a, b))
    for name in ("loader", "logform"):
        Y, F = [], []
        for i in idx:
            r = rows[i]
            x = mpmath.mpf(r["x"])
            y = mpmath.mpf(1.0 - r["x"])
            if name == "loader":
                v = M.density(r["x"], a, b, {"stir": "exact"})
            else:
                v = L.exp_v6(L.R64((a - 1) * mpmath.log(x) + (b - 1) * mpmath.mpf(L.LN(float(y))) - lnB))
            Y.append(L.err_units(v, r["exact"]))
            F.append([1.0, float((y - (1 - x)) / (1 - x) * U)])
        c, *_ = np.linalg.lstsq(np.array(F), np.array(Y), rcond=None)
        print(f"   {blk} ({a:g},{b:g}) synthetic {name}: coef {c[1]:+.2f}  (loader pred {(b-1)-(a+b-2)*(1-r0['x']):+.2f},"
              f" logform pred {b-1:+.2f})", flush=True)

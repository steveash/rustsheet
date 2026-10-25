"""Step view: per run, R_k = ln(Excel_k x_k) - P_k and D_k = R_k - (K' - h log1p(r_exact)) in units, with the grids
ulp(K'+H), ulp(P), ulp(S2).  Usage: s0_steps.py SRC tag [maxrows]"""
import sys

import mpmath

import r5lib as R

src, tag = sys.argv[1], sys.argv[2]
mx_rows = int(sys.argv[3]) if len(sys.argv) > 3 else 60
for blk, rs, m, n in R.PL.runs(src, 0, tag):
    out = []
    for r in rs[:mx_rows]:
        x = r["x"]
        s1, a, b, h, rr, P = R.base(x, m, n)
        X = R.mpf(x)
        rex = m * X / n if s1 else n / (m * X)
        Kp = R.s3_order.kprime(m, n, s1)
        H = -h * mpmath.log1p(rex)
        tau = mpmath.log(R.mpf(r["excel"])) + mpmath.log(X)
        D = float((tau - P - Kp - H) * R.U)
        out.append((x, rr, D, float(Kp + H), float(P)))
    x0, r0, _, S1, P0 = out[0]
    print(f"{blk} side{1 if R.base(x0, m, n)[0] else 2} r={r0:.4f} K+H={S1:.2f} (ulp {R.ulpu(S1):.0f}) "
          f"P={P0:.2f} (ulp {R.ulpu(P0):.0f}) S2={S1 + P0:.2f} (ulp {R.ulpu(S1 + P0):.0f})")
    print("  D:", " ".join(f"{d:.0f}" for _, _, d, _, _ in out))

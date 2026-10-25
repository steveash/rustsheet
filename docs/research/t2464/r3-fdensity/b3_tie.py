"""b3: argument-rounding response.  Per block, D = ln(Excel) - ln(exact F at x) (units) is regressed on the relative
rounding errors (units) of the computed transformed point p and complement q, for each way of forming them (R df:
f0 = 1/(n + m x), q = n f0, p = m x f0; div: p = m x/(m x + n), q = n/(m x + n); w5: p = 1/(1 + n/(m x)),
q = 1/(1 + m x/n)).  If Excel's F is an accurate function of its computed (p, q) as p^alpha q^beta ..., the slopes
are (alpha, beta): beta density times y q/x gives (a, b); R df gives (a - 1, b + 1).
Usage: b3_tie.py [SRC]"""
import collections
import sys

import mpmath
import numpy as np

import flib as F

mpf = mpmath.mpf
U = mpf(2) ** 53


def eps(x, m, n, form):
    X, M, N = mpf(x), mpf(m), mpf(n)
    pe, qe = M * X / (M * X + N), N / (M * X + N)
    if form == "R":
        p, q, _ = F.df_pq(x, m, n, {"pq": "R"})
    else:
        p, q = F.yq(x, m, n, form)
    ep = float((mpf(float(p)) - pe) / pe * U)
    eq = float((mpf(float(q)) - qe) / qe * U)
    return ep, eq


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else "probe:a"
    if src == "tdens":
        rows = F.tdens_rows()
    else:
        import mkprobe

        rows = mkprobe.rows(src.split(":", 1)[1])
    rows = F.with_exact(rows)
    g = collections.defaultdict(list)
    for r in rows:
        g[(r["d1"], r["d2"], r["blk"])].append(r)
    print("block | D sd | per form: alpha beta (residual sd)   | (a-1, b-1)")
    for key, rs in sorted(g.items()):
        if len(rs) < 32:
            continue
        m, n = key[0], key[1]
        Y = np.array([float((mpmath.log(r["excel"]) - mpmath.log(r["exact"])) * U) for r in rs])
        out = []
        for form in ("R", "div", "w5"):
            E = np.array([eps(r["x"], m, n, form) for r in rs])
            A = np.column_stack([E, np.ones(len(rs))])
            c, *_ = np.linalg.lstsq(A, Y, rcond=None)
            res = Y - A @ c
            out.append(f"{form}: {c[0]:+7.2f} {c[1]:+7.2f} ({res.std():5.2f})")
        print(f"{m:g},{n:g} {key[2][-6:]:6s} | {Y.std():6.2f} | " + " | ".join(out) + f" | ({m / 2 - 1:g}, {n / 2 - 1:g})")


if __name__ == "__main__":
    main()

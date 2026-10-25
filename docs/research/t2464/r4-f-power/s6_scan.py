"""s6: greedy scan for extra stored products.  Base regressors rx, eLx, eL (s2).  Extra candidates: own rounding of
fl(c*LN(v)) (natural log, Excel LN) and of fl(c*log2(v)) (x87 FYL2X style, residue scaled by ln 2) for c in a set of
shape coefficients and v in the stored intermediates.  For each run, the best single extra regressor by residual
sd; then a tally of which candidate wins how often.  Usage: s6_scan.py SRC MINMAX"""
import collections
import sys

import mpmath
import numpy as np

import cands as C
import plib as P
import s2_reg

LDB, S, ex = C.LDB, C.S, C.ex
mpf = mpmath.mpf
U = P.U
LN2 = mpmath.log(2)


def extra(x, m, n):
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    mx, r1, r2, den = s["mx"], s["r1"], s["r2"], s["den"]
    vs = {"x": x, "mx": mx, "r1": r1, "r2": r2, "s1": C.st(1 + LDB(r1)), "s2": C.st(1 + LDB(r2)), "den": den,
          "y": C.st(LDB(mx) / LDB(den)), "q": C.st(LDB(n) / LDB(den))}
    cs = {"a": a, "b": b, "h": h, "a-1": a - 1, "b+1": b + 1, "a+1": a + 1, "b-1": b - 1, "h-1": h - 1}
    out = {}
    for vn, v in vs.items():
        Lv = C.LNx(v)
        l2 = mpmath.log(mpf(v)) / LN2
        for cn, c in cs.items():
            if c == 0:
                continue
            p = LDB(c) * Lv
            out[f"{cn}*LN({vn})"] = float((ex(S(p)) - ex(p)) * U)
            e2 = mpf(c) * l2
            out[f"{cn}*log2({vn})"] = float((mpf(float(e2)) - e2) * U * LN2)
    return out


def base_fit(rs, m, n):
    """D, the design of the s2 base regressors (rx, eLx, eL, constant) and the base residual sd."""
    D = np.array([P.D_units(r) for r in rs])
    F0 = np.array([[s2_reg.feats(r["x"], m, n)[0][k] for k in ("rx", "eLx", "eL")] for r in rs])
    ok = [j for j in range(3) if F0[:, j].std() > 0.01]
    B = np.column_stack([F0[:, ok], np.ones(len(D))])
    cf, *_ = np.linalg.lstsq(B, D, rcond=None)
    return D, B, (D - B @ cf).std()


def scan_extra(D, B, E, min_sd):
    """(residual sd, name, coefficient) for each extra regressor of E added alone to the base design, best first."""
    res = []
    for k in E[0]:
        e = np.array([d[k] for d in E])
        if e.std() < min_sd:
            continue
        A = np.column_stack([B, e])
        c2, *_ = np.linalg.lstsq(A, D, rcond=None)
        res.append(((D - A @ c2).std(), k, c2[-1]))
    return sorted(res)


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    wins = collections.Counter()
    for blk, rs, m, n in P.runs(src, minmax):
        D, B, base = base_fit(rs, m, n)
        res = scan_extra(D, B, [extra(r["x"], m, n) for r in rs], 0.5)
        wins[res[0][1]] += 1
        print(f"{blk:20s} base {base:6.1f} | " + " | ".join(f"{k} {sd:.1f} ({c:+.2f})" for sd, k, c in res[:4]))
    print(wins.most_common(10))


if __name__ == "__main__":
    main()

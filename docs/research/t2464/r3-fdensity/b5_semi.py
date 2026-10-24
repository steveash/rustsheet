"""b5: semi-exact models.  F_model = exact F at x times exp(-h * e_s), e_s = relative error of the stored
s = fl(1 + r) against the exact 1 + r, where r is n/(m x) (s1) or m x/n (s2), computed from mx = fl(m x) (x87
stored ops): Excel's dominant amplified rounding (b4).  Variants choose which s by a switch rule.  Per block: sd of
D - prediction (units), and exact / 4 ulp / 15-digit counts of the semi-exact value.
Usage: b5_semi.py [SRC]"""
import collections
import statistics as stt
import sys

import mpmath

import flib as F

mpf = mpmath.mpf
U = mpf(2) ** 53


def es(x, m, n, which):
    X, M, N = mpf(x), mpf(m), mpf(n)
    mx = mpf(float(M * X))
    if which == 1:
        r = mpf(float(N / mx))
        ex = 1 + N / (M * X)
    else:
        r = mpf(float(mx / N))
        ex = 1 + M * X / N
    sv = mpf(float(1 + r))
    return (sv - ex) / ex


RULES = {
    "none": lambda x, m, n: 0,
    "s1": lambda x, m, n: 1,
    "s2": lambda x, m, n: 2,
    "smaller-r": lambda x, m, n: 1 if m * x >= n else 2,
    "larger-r": lambda x, m, n: 2 if m * x >= n else 1,
}


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else "probe:a"
    if src == "cells":
        rows = F.cells_rows()
    else:
        import mkprobe

        rows = mkprobe.rows(src.split(":", 1)[1])
    rows = F.with_exact(rows)
    for nm, rule in RULES.items():
        sds = []
        g = collections.defaultdict(list)
        pairs = []
        for r in rows:
            g[(r["d1"], r["d2"], r["blk"])].append(r)
        for key, rs in g.items():
            m, n = key[0], key[1]
            h = (m + n) / 2
            res = []
            for r in rs:
                w = rule(r["x"], m, n)
                e = es(r["x"], m, n, w) if w else 0
                pred = r["exact"] * mpmath.exp(-h * mpmath.log1p(e)) if w else r["exact"]
                D = (mpmath.log(r["excel"]) - mpmath.log(pred)) * U
                res.append(float(D))
                pairs.append((float(pred), r["excel"]))
            if len(rs) >= 8:
                sds.append(stt.pstdev(res))
        print(f"{nm:10s} median block sd {stt.median(sds):6.2f}  mean block sd {stt.mean(sds):6.2f}  {F.fmt(F.tally(pairs))}")


if __name__ == "__main__":
    main()

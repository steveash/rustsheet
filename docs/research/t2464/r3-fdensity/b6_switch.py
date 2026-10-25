"""b6: per block, the slope of D on e_s1 and e_s2 (b5's total rounding errors of fl(1 + n/(m x)) and
fl(1 + m x/n)), fitted together, against -h; and the ratios m x/n.  Finds which s Excel uses where.
Usage: b6_switch.py [SRC]"""
import collections
import sys

import mpmath
import numpy as np

import b5_semi as B
import flib as F

U = mpmath.mpf(2) ** 53


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else "probe:a"
    import mkprobe

    rows = F.with_exact(mkprobe.rows(src.split(":", 1)[1]))
    g = collections.defaultdict(list)
    for r in rows:
        g[(r["d1"], r["d2"], r["blk"])].append(r)
    for key, rs in sorted(g.items()):
        m, n = key[0], key[1]
        h = (m + n) / 2
        D = np.array([float((mpmath.log(r["excel"]) - mpmath.log(r["exact"])) * U) for r in rs])
        E = np.array([[float(B.es(r["x"], m, n, w) * U) for w in (1, 2)] for r in rs])
        A = np.column_stack([E, np.ones(len(rs))])
        c, *_ = np.linalg.lstsq(A, D, rcond=None)
        res = D - A @ c
        mxn = m * rs[0]["x"] / n
        y = mxn / (1 + mxn)
        print(f"{m:g},{n:g} {key[2][-6:]:6s} mx/n {mxn:7.3f} y {y:.3f} | s1 {c[0] / -h:+.2f}h s2 {c[1] / -h:+.2f}h "
              f"(sd e {E[:, 0].std():.2f} {E[:, 1].std():.2f}) | D sd {D.std():6.2f} -> {res.std():6.2f}")


if __name__ == "__main__":
    main()

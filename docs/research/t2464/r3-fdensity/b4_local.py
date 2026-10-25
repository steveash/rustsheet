"""b4: local-rounding regression.  Per block, D = ln(Excel) - ln(exact F) (units) is regressed on the local rounding
error (units, relative) of each candidate intermediate op, every op computed from the stored results before it:
  mx = m*x;  w5: r1 = n/mx, s1 = 1 + r1, y = 1/s1;  r2 = mx/n, s2 = 1 + r2, q = 1/s2;
  div: d = n + mx, pd = mx/d, qd = n/d;  R df: f0 = 1/d, qR = n*f0, pR = mx*f0.
A path Excel uses shows its sensitivities (e.g. lnF = a ln y + b ln q - ln x gives +a on y, -a on s1, -b on s2, +b
on q).  Printed: coefficient/(predicted sensitivity scale) per op, and the residual sd.
Usage: b4_local.py [SRC] [min a+b]"""
import collections
import sys

import mpmath
import numpy as np

import flib as F

mpf = mpmath.mpf
U = mpf(2) ** 53
OPS = ["mx", "r1", "s1", "y", "r2", "s2", "q", "d", "pd", "qd", "f0", "qR", "pR"]


def local(x, m, n):
    X, M, N = mpf(x), mpf(m), mpf(n)
    v = {}
    e = {}

    def op(name, exact):
        st = float(exact)
        v[name] = mpf(st)
        e[name] = float((v[name] - exact) / exact * U)

    op("mx", M * X)
    op("r1", N / v["mx"])
    op("s1", 1 + v["r1"])
    op("y", 1 / v["s1"])
    op("r2", v["mx"] / N)
    op("s2", 1 + v["r2"])
    op("q", 1 / v["s2"])
    op("d", N + v["mx"])
    op("pd", v["mx"] / v["d"])
    op("qd", N / v["d"])
    op("f0", 1 / v["d"])
    op("qR", N * v["f0"])
    op("pR", v["mx"] * v["f0"])
    return [e[k] for k in OPS]


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else "probe:a"
    mins = float(sys.argv[2]) if len(sys.argv) > 2 else 0
    import mkprobe

    rows = F.with_exact(mkprobe.rows(src.split(":", 1)[1]))
    g = collections.defaultdict(list)
    for r in rows:
        g[(r["d1"], r["d2"], r["blk"])].append(r)
    print("block | D sd -> res sd | coefficients " + " ".join(OPS))
    for key, rs in sorted(g.items()):
        m, n = key[0], key[1]
        if (m + n) / 2 < mins:
            continue
        Y = np.array([float((mpmath.log(r["excel"]) - mpmath.log(r["exact"])) * U) for r in rs])
        E = np.array([local(r["x"], m, n) for r in rs])
        use = [j for j in range(len(OPS)) if E[:, j].std() > 0.05]
        A = np.column_stack([E[:, use], np.ones(len(rs))])
        c, *_ = np.linalg.lstsq(A, Y, rcond=None)
        res = Y - A @ c
        cc = dict(zip([OPS[j] for j in use], c))
        print(f"{m:g},{n:g} {key[2][-6:]:6s} | {Y.std():6.2f} -> {res.std():6.2f} | "
              + " ".join(f"{k}:{cc[k]:+.1f}" if k in cc else f"{k}:." for k in OPS) + f" | a={m / 2:g} b={n / 2:g}")


if __name__ == "__main__":
    main()

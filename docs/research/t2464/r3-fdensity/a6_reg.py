"""a6: chain-free regression.  D = ln(Excel) - ln(exact F) in units of 2^-53, regressed per block (block constant
removed) on the rounding errors of candidate stored terms (computed term - exact term, signed as each enters lnF).
A coefficient near 1 means Excel stores that term; near 0 means it does not.
Usage: a6_reg.py SRC [min_block]   SRC: tdens | probe:<name>"""
import collections
import sys

import mpmath
import numpy as np

import flib as F

LDB, s = F.LDB, F.st
mpf = mpmath.mpf
U = mpf(2) ** 53


def S(v):
    return LDB(s(v))


def ex(v):
    if isinstance(v, mpmath.mpf):
        return v
    p, q = LDB(v).as_integer_ratio()
    return mpf(p) / q


def feats(x, m, n):
    """{name: rounding error (units) of the computed term minus the exact term, signed as in lnF}."""
    a, b, h = m / 2, n / 2, (m + n) / 2
    X, M, N = mpf(x), mpf(m), mpf(n)
    mx = s(LDB(m) * LDB(x))
    d = s(LDB(mx) + LDB(n))
    r = s(LDB(mx) / LDB(n))
    out = {}

    def add(name, comp, exact, sign):
        out[name] = float((ex(comp) - exact) * U) * sign

    add("h*LN(n+mx)", S(LDB(h) * F.LN(d)), h * mpmath.log(N + M * X), -1)
    add("h*LN(1+mx/n)", S(LDB(h) * F.LN(s(1 + LDB(r)))), h * mpmath.log1p(M * X / N), -1)
    add("h*l1p(mx/n)", S(LDB(h) * LDB(F.L.log1p_xl(r))), h * mpmath.log1p(M * X / N), -1)
    add("a*LN(y)", S(LDB(a) * F.LN(s(LDB(mx) / LDB(d)))), a * mpmath.log(M * X / (N + M * X)), 1)
    add("b*LN(q)", S(LDB(b) * F.LN(s(LDB(n) / LDB(d)))), b * mpmath.log(N / (N + M * X)), 1)
    add("(a-1)*LN(x)", S(LDB(a - 1) * F.LN(x)), (a - 1) * mpmath.log(X), 1)
    add("LN(x)", F.LN(x), mpmath.log(X), -1)
    add("a*LN(mx)", S(LDB(a) * F.LN(mx)), a * mpmath.log(M * X), 1)
    add("a*LN(mx/n)", S(LDB(a) * F.LN(r)), a * mpmath.log(M * X / N), 1)
    # products of the ratio model (c1) with the log computed in other ways: extended FYL2X, exact, Excel log1p
    cx = a - 1 if mx <= n else -(b + 1)
    rr = r if mx <= n else s(LDB(n) / LDB(mx))
    lx = mpmath.log(X)
    add("cx*ln64(x)", S(LDB(cx) * F.R3.LN64(x)), cx * lx, 1)
    add("cx*lnx exact", mpf(float(cx * lx)), cx * lx, 1)
    l1 = mpmath.log1p(mpf(rr))
    add("h*l1p_m(r)", mpf(float(h * l1)), h * mpmath.log1p(M * X / N if mx <= n else N / (M * X)), -1)
    add("h*l1p_xl(r)", S(LDB(h) * LDB(F.TD.log1p_xl(rr))), h * mpmath.log1p(M * X / N if mx <= n else N / (M * X)), -1)
    return out


def main():
    src = sys.argv[1]
    minb = int(sys.argv[2]) if len(sys.argv) > 2 else 32
    if src == "tdens":
        rows = F.tdens_rows()
    else:
        import mkprobe

        rows = mkprobe.rows(src.split(":", 1)[1])
    rows = F.with_exact(rows)
    g = collections.defaultdict(list)
    for r in rows:
        g[(r["d1"], r["d2"], r["blk"])].append(r)
    for key, rs in sorted(g.items()):
        if len(rs) < minb:
            continue
        fs = [feats(r["x"], r["d1"], r["d2"]) for r in rs]
        names = list(fs[0])
        Y = np.array([float((mpmath.log(mpf(r["excel"])) - mpmath.log(r["exact"])) * U) for r in rs])
        X = np.array([[f[k] for k in names] for f in fs])
        sd = X.std(axis=0)
        cols = [j for j in range(len(names)) if sd[j] > 0.05]
        A = np.column_stack([X[:, cols] - X[:, cols].mean(axis=0), np.ones(len(rs))])
        c, *_ = np.linalg.lstsq(A, Y, rcond=None)
        res = Y - A @ c
        line = " ".join(f"{names[j]}={c[i]:+.2f}(sd{sd[j]:.1f})" for i, j in enumerate(cols))
        print(f"{key[0]:g},{key[1]:g} {key[2][:22]:22s} n={len(rs)} Ysd {Y.std():5.2f} -> res {res.std():5.2f} | {line}")
    F.save_cache()


if __name__ == "__main__":
    main()

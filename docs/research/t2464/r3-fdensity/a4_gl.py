"""a4: direct log-gamma compositions of the F density (Excel GAMMALN and LN, x87 stored ops), against Excel:
within-(d1,d2) sd and mean of (model - Excel) in units, and exact / 4 ulp / 15-digit counts.
lnF = lnG + a ln(m/n) + (a-1) ln x - h ln(1 + m x/n),  lnG = GL(h) - GL(a) - GL(b),  a = m/2, b = n/2, h = a + b.
Usage: a4_gl.py [form ...]"""
import collections
import statistics as st
import sys

import flib as F

LDB, s = F.LDB, F.st


def S(v):  # store
    return LDB(s(v))


def gpart(a, b, h, form):
    g = form.get("g", "hab")
    if g == "hab":
        return S(S(LDB(F.GL(h)) - LDB(F.GL(a))) - LDB(F.GL(b)))
    if g == "hba":
        return S(S(LDB(F.GL(h)) - LDB(F.GL(b))) - LDB(F.GL(a)))
    if g == "lnB":  # -(GL(a) + GL(b) - GL(h))
        return -S(S(LDB(F.GL(a)) + LDB(F.GL(b))) - LDB(F.GL(h)))
    if g == "lnBt":  # TOMS-like: GL(a) + (GL(b) - GL(h))
        return -S(LDB(F.GL(a)) + S(LDB(F.GL(b)) - LDB(F.GL(h))))
    if g == "exact":
        import mpmath

        return F.R3._ld_of(mpmath.loggamma(h) - mpmath.loggamma(a) - mpmath.loggamma(b))
    raise ValueError(g)


def lnF(x, m, n, form):
    a, b, h = m / 2, n / 2, (m + n) / 2
    X, M, N = LDB(x), LDB(m), LDB(n)
    G = gpart(a, b, h, form)
    t = form.get("t", "A")
    if t == "A":  # a ln(m/n) + (a-1) ln x - h ln(1 + m x/n)
        t1 = S(LDB(a) * F.LN(s(M / N)))
        t2 = S(LDB(a - 1) * F.LN(x))
        t3 = S(LDB(h) * F.LN(s(1 + S(S(M * X) / N))))
        terms = [G, t1, t2, -t3]
    elif t == "B":  # a ln m + b ln n + (a-1) ln x - h ln(n + m x)
        terms = [G, S(LDB(a) * F.LN(m)), S(LDB(b) * F.LN(n)), S(LDB(a - 1) * F.LN(x)),
                 -S(LDB(h) * F.LN(s(N + S(M * X))))]
    elif t == "C":  # a ln y + b ln(1-y) - ln x, y = m x/(m x + n), 1-y = n/(m x + n)
        d = S(S(M * X) + N)
        terms = [G, S(LDB(a) * F.LN(s(S(M * X) / d))), S(LDB(b) * F.LN(s(N / d))), -F.LN(x)]
    elif t == "D":  # a ln(m x) + b ln n - h ln(m x + n) - ln x
        mx = S(M * X)
        terms = [G, S(LDB(a) * F.LN(mx)), S(LDB(b) * F.LN(n)), -S(LDB(h) * F.LN(s(mx + N))), -F.LN(x)]
    elif t == "E":  # a ln(m x/n) - h ln(1 + m x/n) - ln x
        r = S(S(M * X) / N)
        terms = [G, S(LDB(a) * F.LN(r)), -S(LDB(h) * F.LN(s(1 + r))), -F.LN(x)]
    else:
        raise ValueError(t)
    acc = form.get("acc", "x")
    E = terms[0]
    for v in terms[1:]:
        E = E + v
        if acc == "x":
            E = S(E)
    return E


def value(x, m, n, form):
    return F.R3.exp_v6(lnF(x, m, n, form))


FORMS = {f"{t}-{g}-{acc}": dict(t=t, g=g, acc=acc) for t in "ABCDE" for g in ("hab", "lnB", "lnBt", "exact")
         for acc in ("x", "e")}


def main():
    rows = F.with_exact(F.cells_rows() + F.tdens_rows())
    rows = [r for r in rows if not (r["src"] == "tdens-a2" and r["blk"].startswith("df"))]
    F.gl_prefetch({v for r in rows for v in (r["d1"] / 2, r["d2"] / 2, (r["d1"] + r["d2"]) / 2)})
    names = sys.argv[1:] or list(FORMS)
    for nm in names:
        o = FORMS[nm]
        dev = collections.defaultdict(list)
        g = collections.defaultdict(list)
        for r in rows:
            v = value(r["x"], r["d1"], r["d2"], o)
            dev[(r["d1"], r["d2"])].append(F.units(v, r["excel"]))
            g[r["src"].split("-")[0]].append((v, r["excel"]))
        sds = [st.pstdev(v) for v in dev.values() if len(v) >= 8]
        big = {k: v for k, v in dev.items() if k[0] >= 20 and k[1] >= 20 and len(v) >= 8}
        print(f"{nm:14s} median sd {st.median(sds):6.2f}  big-shape sd "
              + " ".join(f"{k[0]:g},{k[1]:g}:{st.pstdev(v):.1f}" for k, v in sorted(big.items()))
              + " | " + " | ".join(f"{k} {F.fmt(F.tally(v))}" for k, v in sorted(g.items())))
        sys.stdout.flush()


if __name__ == "__main__":
    main()

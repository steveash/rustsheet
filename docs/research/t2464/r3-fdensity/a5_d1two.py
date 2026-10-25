"""a5: d1 = 2, where R df's dbinom_raw takes its x = 0 branch: F(x; 2, n) = q * q^(n/2) in exact arithmetic,
q = n/(n + 2x).  Scores (exact / 4 ulp / 15 digits) of closed-form variants on every d1 = 2 cell of cells.tsv.
Usage: a5_d1two.py"""
import collections
import itertools

import flib as F

LDB, s = F.LDB, F.st


def S(v):
    return LDB(s(v))


def qform(x, n, qf):
    X, N = LDB(x), LDB(n)
    if qf == "R":  # f0 = 1/(n + 2x); q = n f0; p = 2x f0
        f0 = S(1 / S(N + S(2 * X)))
        return S(N * f0), S(S(2 * X) * f0)
    if qf == "div":
        d = S(N + S(2 * X))
        return S(N / d), S(S(2 * X) / d)
    if qf == "w5":
        return S(1 / S(1 + S(S(2 * X) / N))), S(1 / S(1 + S(N / S(2 * X))))
    raise ValueError(qf)


def value(x, n, o):
    q, p = qform(x, n, o["q"])
    N = LDB(n / 2)
    lg = o["lg"]
    if lg == "LNq":
        lc = N * F.LN(q)
    elif lg == "LNq1":  # (n/2 + 1) LN q: the prefactor folded into the power
        lc = (N + 1) * F.LN(q)
    elif lg == "l1p":  # n/2 log1p(-p) with Excel's log1p (atanh series to 0.375, LN(1-p) above)
        lc = N * LDB(F.L.log1p_xl(-float(p)))
    elif lg == "l1p1":
        lc = (N + 1) * LDB(F.L.log1p_xl(-float(p)))
    elif lg == "bd0":  # R: p < 0.1 -> -bd0(N, N q) - N p
        if p < 0.1:
            b, _ = F.R3.bd0(float(N), S(N * q), {"v": "x", "ph": 1, "ld": "x"})
            lc = -b - S(N * p)
        else:
            lc = N * F.LN(q)
    lc = S(lc) if o["lcs"] == "x" else lc
    if lg.endswith("1"):
        return F.R3.exp_v6(lc)
    d = F.R3.exp_v6(lc, True)
    d = S(d) if o["ds"] == "x" else d
    return s(q * d)


def main():
    rows = [r for r in F.with_exact(F.cells_rows()) if r["d1"] == 2]
    res = []
    for qf, lg, lcs, ds in itertools.product(("R", "div", "w5"), ("LNq", "LNq1", "l1p", "l1p1", "bd0"), "xe", "xe"):
        o = dict(q=qf, lg=lg, lcs=lcs, ds=ds)
        g = collections.defaultdict(list)
        for r in rows:
            g["all"].append((value(r["x"], r["d2"], o), r["excel"]))
            g["p<0.1" if 2 * r["x"] / (r["d2"] + 2 * r["x"]) < 0.1 else "p>=0.1"].append(g["all"][-1])
        res.append((F.tally(g["all"])[1], o, g))
    res.sort(key=lambda t: -t[0])
    print("CR", F.fmt(F.tally([(r["cr"], r["excel"]) for r in rows])))
    for _, o, g in res[:14]:
        print(o, " | ".join(f"{k} {F.fmt(F.tally(v))}" for k, v in sorted(g.items())))


if __name__ == "__main__":
    main()

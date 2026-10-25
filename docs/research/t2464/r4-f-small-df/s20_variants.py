"""In-sample variants of M-sd's final step (constant fixed to sdmodel.K_of): 'div' (M-sd), 'xn64' (E - ln x unstored,
no division), 'mulr' (EXP(E) * fl(1/x)), 'divx' (EXP extended / x). Usage: s20_variants.py SET [SET ...]"""
import sys

import sdmodel as SD
import slib as L

LDB, st = L.LDB, L.st


def val(r, fin):
    x, m, n = r["x"], r["d1"], r["d2"]
    s, rr = L.side_r(x, m, n)
    a, b, h = m / 2, n / 2, (m + n) / 2
    c = a if s == 1 else -b
    E = SD.K_of(m, n, s) + LDB(st(LDB(c) * L.LN(x))) - LDB(h) * L.L_xl(rr)
    if fin == "xn64":
        return L.EXP(E - L.R3.LN64(x))
    if fin == "mulr":
        return st(LDB(L.EXP(E)) * LDB(st(LDB(1) / LDB(x))))
    if fin == "divx":
        return st(L.R3.exp_v6(E, ext=True) / LDB(x))
    return st(LDB(L.EXP(E)) / LDB(x))


for s in sys.argv[1:]:
    rows = [r for r in L.load(s) if max(r["d1"], r["d2"]) <= 10]
    for fin in ("div", "xn64", "mulr", "divx"):
        print(s, fin, L.F.fmt(L.score_rows(rows, lambda r: val(r, fin))["all"]))

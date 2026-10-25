"""(2,2), K = 0 exactly: an all-double ("SSE2 / CRT") family against the x87 family, exact-match counts per side.
LN: 'xl' Excel LN | 'cr' correctly rounded log; L: 'xl' Excel log1p | 'cr' CR log1p | 'crln' CR log(fl(1+r));
EXP: 'v6' Excel EXP | 'cr' CR exp; E: 'st' stored double | 'ext' extended; final: 'div' fl(EXP/x) | 'none'
(the ln x terms cancel: E = -2 L) | 'xn' E = P - lnx(-2L) with the second ln stored."""
import itertools

import slib as L

LDB, st, mp = L.LDB, L.st, L.mpmath
rows = [r for r in L.load("mya") if (r["d1"], r["d2"]) == (2.0, 2.0)]


def crf(f, v):
    return float(f(L.mpf(v)))


def fr(v):
    p, q = LDB(v).as_integer_ratio()
    return L.mpf(p) / q


res = {}
for r in rows:
    x = r["x"]
    s, rr = L.side_r(x, 2.0, 2.0)
    c = 1.0 if s == 1 else -1.0
    for lnm, lm, em, es, fin in itertools.product(("xl", "cr"), ("xl", "cr", "crln"), ("v6", "cr"), ("st", "ext"),
                                                  ("div", "none", "xn")):
        lnx = LDB(L.LN(x)) if lnm == "xl" else LDB(crf(mp.log, x))
        if lm == "xl":
            Lr = L.L_xl(rr)
        elif lm == "cr":
            Lr = LDB(crf(mp.log1p, rr))
        else:
            Lr = LDB(crf(mp.log, st(LDB(1) + LDB(rr))))
        P = LDB(st(LDB(c) * lnx))
        if fin == "none":
            E = -2 * Lr
        elif fin == "xn":
            E = P - 2 * Lr - lnx if es == "ext" else LDB(st(LDB(st(P - 2 * Lr)) - lnx))
        else:
            E = P - 2 * Lr
        if es == "st":
            E = LDB(st(E))
        v = L.R3.exp_v6(E) if em == "v6" else float(mp.exp(fr(E)))
        if fin == "div":
            v = st(LDB(v) / LDB(x))
        k = (s, lnm, lm, em, es, fin)
        res[k] = res.get(k, 0) + (v == r["excel"])
for k, v in sorted(res.items(), key=lambda kv: -kv[1])[:25]:
    print(k, v, "of 64")

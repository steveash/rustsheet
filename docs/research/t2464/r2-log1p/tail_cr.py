"""Control models for the tail: s = RN(atanh(r) - r) (correctly rounded tail) and s = fwd-sum model, both with
l = 2*fl(r + s) x87; plus the admitted s window per fwd-miss row.  Usage: tail_cr.py"""
import math

import mpmath

import coef_scan as C
import gen_scan as G
import r2lib as R
from x87f import x87 as X


def s_fwd(r, N=14):
    t2 = X("*", r, r)
    p, s = r, None
    for k in range(1, N + 1):
        p = X("*", p, t2)
        tm = X("*", p, C.BASE[k])
        s = tm if s is None else X("+", s, tm)
    return s


def s_cr(r):
    m = R.mpf(r)
    return float(mpmath.atanh(m) - m)


rows = G.load_rows()
cnt = {"cr": [0, 0], "fwd": [0, 0]}
for p, r, l, h in rows:
    sc, sf = s_cr(r), s_fwd(r)
    cnt["cr"][h[0]] += 2 * X("+", r, sc) == l
    cnt["fwd"][h[0]] += 2 * X("+", r, sf) == l
    if 2 * X("+", r, sf) != l:
        ok = [k for k in range(-6, 7) if 2 * X("+", r, R.nb(sf, k)) == l]
        print(f"p={p:.10f} -r={-r:.5f} fwd s - CR s = {(sf - sc) / math.ulp(sc):+.0f} ulp; admitted s-fwd offsets {ok}")
print("even/odd exact:", cnt, "of", sum(1 for x in rows if x[3][0] == 0), sum(1 for x in rows if x[3][0] == 1))

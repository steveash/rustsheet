"""Pair test (B-free), finer: x-term and y-term variants including 1-x held exactly in an x87 register.

q = excel / (px*py) agreeing across the x values of one (case, a, b). Per case: median |dq| and share <= 1.5.
"""

import collections
import itertools
import sys

import mpmath

from lib import D, R64, exp_v6, exp_v6_ext, ln64, load_pdf, mpf, xl

mpmath.mp.prec = 200
rows = load_pdf()
N = len(rows)
X = [r["x"] for r in rows]
A = [r["a"] for r in rows]
Bv = [r["b"] for r in rows]
LNx = xl("LN", X)
LNy = xl("LN", [1.0 - x for x in X])


def pxforms(i):
    x, a = mpf(X[i]), mpf(A[i])
    am1 = D(a - 1)
    lx = mpf(LNx[i])
    return {
        "EXP(d(am1*LN))": mpf(exp_v6(D(am1 * lx))),
        "EXP(e(am1*LN))": mpf(exp_v6(R64(am1 * lx))),
        "EXPext(e(am1*LNe))": exp_v6_ext(R64(am1 * ln64(x))),
        "EXP(d(am1*LNe))": mpf(exp_v6(D(am1 * ln64(x)))),
        "exact": x ** (a - 1),
    }


def pyforms(i):
    x, b = mpf(X[i]), mpf(Bv[i])
    bm1 = D(b - 1)
    ye = 1 - x  # exact (x87 register holds it: 1 - x needs at most 64 bits here)
    l1p = mpmath.log1p(-x)
    out = {
        "EXP(d(bm1*D(LN64(1-x))))": mpf(exp_v6(D(bm1 * D(ln64(ye))))),
        "EXP(d(bm1*LN64(1-x)))": mpf(exp_v6(D(bm1 * ln64(ye)))),
        "EXP(e(bm1*LN64(1-x)))": mpf(exp_v6(R64(bm1 * ln64(ye)))),
        "EXPext(e(bm1*LN64(1-x)))": exp_v6_ext(R64(bm1 * ln64(ye))),
        "EXP(d(bm1*log1p_cr))": mpf(exp_v6(D(bm1 * D(l1p)))),
        "EXP(d(bm1*log1p))": mpf(exp_v6(D(bm1 * l1p))),
        "EXP(bm1*log1p exact)": mpf(exp_v6(bm1 * l1p)),
        "exact": ye ** (b - 1),
        "exp_cr(d(bm1*log1p_cr))": mpf(D(mpmath.exp(D(bm1 * D(l1p))))),
    }
    return out


PX = [pxforms(i) for i in range(N)]
PY = [pyforms(i) for i in range(N)]
excel = [mpf(r["excel"]) for r in rows]
groups = collections.defaultdict(list)
for i, r in enumerate(rows):
    groups[(r["case"], A[i], Bv[i])].append(i)
cases = sorted({k[0] for k in groups})
only = sys.argv[1:] or cases
res = []
for kx, ky in itertools.product(PX[0], PY[0]):
    tot, good, per = 0, 0, []
    for c in cases:
        if c not in only:
            continue
        d = []
        for key, g in groups.items():
            if key[0] != c or len(g) < 2:
                continue
            q = [excel[i] / (PX[i][kx] * PY[i][ky]) for i in g]
            d += [float((q[j] - q[0]) / q[0]) * 2**53 for j in range(1, len(q))]
        if d:
            med = sorted(abs(v) for v in d)[len(d) // 2]
            g15 = sum(abs(v) <= 1.5 for v in d)
            per.append(f"{c[-12:]}:{med:.2f},{g15}/{len(d)}")
            tot += len(d)
            good += g15
    res.append((good, kx, ky, per, tot))
res.sort(reverse=True)
for good, kx, ky, per, tot in res:
    print(f"{good}/{tot} {kx} | {ky}\n     " + "  ".join(per))

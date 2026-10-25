"""Single-EXP density forms EXP(E), E = tx + ty - lnB, every storage pattern (each op stored to a double or kept
in an 80-bit register), three groupings, with lx = LN(x) (stored or extended), ly = LN of 1-x (rounded y,
or 1-x exact in a register; stored or extended), lnB = ga + (gb - gab) from Excel's GAMMALN.
Region: argv[1] in {'a<1,b<=8', 'all', ...}. Prints the top patterns, split by case and by hold-out half.
"""

import itertools
import math
import sys

import mpmath

from lib import pdf_region, D, R64, exp_v6, fmt, ln64, load_pdf, mpf, score, xl

rows, X, A, Bv, AB, (GLa, GLb, GLab), want, idx = pdf_region()
N = len(rows)
print(want, len(idx))
ST = {"d": D, "e": R64}
models = {}
for i in idx:
    x, a, b = mpf(X[i]), mpf(A[i]), mpf(Bv[i])
    am1, bm1 = D(a - 1), D(b - 1)
    lxs = {"LNd": mpf(D(ln64(x))), "LNe": ln64(x)}
    lys = {
        "LN(y)d": mpf(D(ln64(mpf(1.0 - X[i])))),
        "LN(1-x)e": ln64(1 - x),
        "LN(1-x)d": mpf(D(ln64(1 - x))),
        "log1p": mpmath.log1p(-x),
    }
    ga, gb, gab = mpf(GLa[i]), mpf(GLb[i]), mpf(GLab[i])
    for slb in "de":
        lnB = ST[slb](ga + ST[slb](gb - gab))
        for (klx, lx), (kly, ly) in itertools.product(lxs.items(), lys.items()):
            for s1, s2, s3, s4 in itertools.product("de", repeat=4):
                tx = ST[s1](am1 * lx)
                ty = ST[s2](bm1 * ly)
                for grp in ("(tx+ty)-L", "tx+(ty-L)", "(tx-L)+ty"):
                    if grp == "(tx+ty)-L":
                        E = ST[s4](ST[s3](tx + ty) - lnB)
                    elif grp == "tx+(ty-L)":
                        E = ST[s4](tx + ST[s3](ty - lnB))
                    else:
                        E = ST[s4](ST[s3](tx - lnB) + ty)
                    name = f"{klx} {kly} lnB{slb} tx{s1} ty{s2} {grp} in{s3} E{s4}"
                    models.setdefault(name, {})[i] = exp_v6(E)
res = []
for name, m in models.items():
    ex = sum(m[i] == rows[i]["excel"] for i in idx)
    res.append((ex, name))
res.sort(reverse=True)
cases = sorted({rows[i]["case"] for i in idx})
for ex, name in res[:30] + [r for r in res if r[1].startswith("LNd LN(1-x)e lnBd") and "tx+(ty-L)" in r[1]]:
    m = models[name]
    per = " ".join(
        f"{c[-10:]}:{sum(m[i] == rows[i]['excel'] for i in idx if rows[i]['case'] == c)}/{sum(1 for i in idx if rows[i]['case'] == c)}"
        for c in cases
    )
    h = [sum(m[i] == rows[i]["excel"] for i in idx if (i % 2) == k) for k in (0, 1)]
    print(f"{ex:4d}/{len(idx)} even {h[0]} odd {h[1]} | {name} | {per}")

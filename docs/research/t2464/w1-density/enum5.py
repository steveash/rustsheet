"""brcomp-style forms: exp(a ln x + b ln y - lnB) / (x y) and (a-1) rewritten as a ln x - ln x, region argv[1]."""

import itertools
import math
import sys

import mpmath

from lib import model_table, pdf_region, D, R64, exp_v6, exp_v6_ext, ln64, load_pdf, mpf, xl

rows, X, A, Bv, AB, (GLa, GLb, GLab), want, idx = pdf_region()
N = len(rows)
ST = {"d": D, "e": R64}
models, put = model_table()


for i in idx:
    x, a, b = mpf(X[i]), mpf(A[i]), mpf(Bv[i])
    am1, bm1 = D(a - 1), D(b - 1)
    ga, gb, gab = mpf(GLa[i]), mpf(GLb[i]), mpf(GLab[i])
    lnB = mpf(D(ga + D(gb - gab)))
    LNx = mpf(D(ln64(x)))
    ly = ln64(1 - x)  # 1-x exact, extended
    lyd = mpf(D(ly))
    ye = 1 - x
    txs = {
        "D(am1*LN)": mpf(D(am1 * LNx)),
        "D(a*LN)-LN e": R64(mpf(D(a * LNx)) - LNx),
        "D(D(a*LN)-LN)": mpf(D(mpf(D(a * LNx)) - LNx)),
        "D(a*LN-LN)": mpf(D(R64(a * LNx) - LNx)),
        "D(LN*am1) e-LN": R64(R64(a * LNx) - LNx),
    }
    tys = {"e(bm1*ly)": R64(bm1 * ly), "D(bm1*ly)": mpf(D(bm1 * ly)), "e(b*ly)-ly": R64(R64(b * ly) - ly)}
    for (kx, tx), (ky, ty) in itertools.product(txs.items(), tys.items()):
        for g in range(3):
            if g == 0:
                E = R64(tx + R64(ty - lnB))
            elif g == 1:
                E = R64(R64(tx + ty) - lnB)
            else:
                E = R64(R64(tx - lnB) + ty)
            put(f"EXP(E) tx={kx} ty={ky} g{g}", i, exp_v6(E))
    # z = a ln x + b ln y - lnB, then divide
    for sa, sb in itertools.product("de", repeat=2):
        za = ST[sa](a * LNx)
        zb = ST[sb](b * ly)
        for g in range(2):
            z = R64(za + R64(zb - lnB)) if g == 0 else R64(R64(za + zb) - lnB)
            p = exp_v6_ext(z)
            pd = mpf(D(p))
            put(f"brcomp za{sa} zb{sb} g{g} /x/y stored", i, D(mpf(D(pd / x)) / ye))
            put(f"brcomp za{sa} zb{sb} g{g} /(x*y)", i, D(pd / R64(x * ye)))
            put(f"brcomp za{sa} zb{sb} g{g} /(x*yd)", i, D(pd / mpf(D(x * mpf(1.0 - X[i])))))
            put(f"brcomp za{sa} zb{sb} g{g} ext/(x*y)", i, D(R64(p / R64(x * ye))))
            put(f"brcomp za{sa} zb{sb} g{g} pd/x/yd", i, D(mpf(D(pd / x)) / mpf(1.0 - X[i])))

res = sorted(((sum(m[i] == rows[i]["excel"] for i in idx), k) for k, m in models.items()), reverse=True)
cases = sorted({rows[i]["case"] for i in idx})
print(want, len(idx))
for ex, k in res[:25]:
    m = models[k]
    per = " ".join(
        f"{c[-10:]}:{sum(m[i] == rows[i]['excel'] for i in idx if rows[i]['case'] == c)}/{sum(1 for i in idx if rows[i]['case'] == c)}"
        for c in cases
    )
    print(f"{ex:4d} | {k} | {per}")

"""Base-2 (x87 pow-style) density forms: FYL2X / FYL2XP1 exponents, F2XM1 tail, against base-e EXP(E) forms.

Region argv[1] (default a<1,b<=8, where B = EXP(GAMMALN(a)+(GAMMALN(b)-GAMMALN(a+b))) is known).
"""

import itertools
import math
import sys

import mpmath

from lib import (
    model_table,
    pdf_region,
    D,
    LOG2E64,
    R64,
    exp2_v6,
    exp2_v6_ext,
    exp_v6,
    exp_v6_ext,
    fyl2x,
    fyl2xp1,
    ln64,
    load_pdf,
    mpf,
    xl,
)

rows, X, A, Bv, AB, (GLa, GLb, GLab), want, idx = pdf_region()
N = len(rows)
ST = {"d": D, "e": R64}
models, put = model_table()


for i in idx:
    x, a, b = mpf(X[i]), mpf(A[i]), mpf(Bv[i])
    am1, bm1 = D(a - 1), D(b - 1)
    ga, gb, gab = mpf(GLa[i]), mpf(GLb[i]), mpf(GLab[i])
    lnB = D(ga + D(gb - gab))
    Bd = mpf(exp_v6(lnB))
    Be = exp_v6_ext(lnB)
    tx2 = fyl2x(am1, x)
    ty2s = {
        "fyl2x(1-x)": fyl2x(bm1, 1 - x),
        "fyl2xp1(-x)": fyl2xp1(bm1, -x),
        "fyl2x(y)": fyl2x(bm1, 1.0 - X[i]),
    }
    L2s = {"lnB*l2e64": R64(lnB * LOG2E64), "lnB/ln2": R64(lnB / mpmath.ln(2))}
    for (ky, ty2), (kl, L2) in itertools.product(ty2s.items(), L2s.items()):
        for s in "de":
            S = ST[s](tx2 + ty2)
            p = exp2_v6_ext(S)
            put(f"2^(tx+ty) {ky} sum{s} ; D(D(p)/B)", i, D(mpf(D(p)) / Bd))
            put(f"2^(tx+ty) {ky} sum{s} ; D(p/B)", i, D(p / Bd))
            put(f"2^(tx+ty) {ky} sum{s} ; D(p/Be)", i, D(p / Be))
            put(f"2^(tx+ty) {ky} sum{s} ; D(D(p)*D(1/B))", i, D(mpf(D(p)) * D(1 / Bd)))
            for g in ("(tx+ty)-L", "tx+(ty-L)", "(tx-L)+ty"):
                if g == "(tx+ty)-L":
                    T = R64(ST[s](tx2 + ty2) - L2)
                elif g == "tx+(ty-L)":
                    T = R64(tx2 + ST[s](ty2 - L2))
                else:
                    T = R64(ST[s](tx2 - L2) + ty2)
                put(f"2^({g}) {ky} {kl} in{s}", i, exp2_v6(T))
        # separate powers each 2^(.) stored
        px = exp2_v6_ext(tx2)
        py = exp2_v6_ext(ty2)
        for s1, s2 in itertools.product("de", repeat=2):
            PX = px if s1 == "e" else mpf(R53x := D(px))
            PY = py if s2 == "e" else mpf(D(py))
            put(f"px{s1}*py{s2} {ky} ; D(D(px*py)/B)", i, D(mpf(D(PX * PY)) / Bd))
            put(f"px{s1}*py{s2} {ky} ; D(e(px*py)/B)", i, D(R64(PX * PY) / Bd))
            put(f"px{s1}*py{s2} {ky} ; D(px*D(py/B))", i, D(PX * mpf(D(PY / Bd))))
            put(f"px{s1}*py{s2} {ky} ; D(D(px/B)*py)", i, D(mpf(D(PX / Bd)) * PY))
            put(f"px{s1}*py{s2} {ky} ; e chain /Be", i, D(R64(R64(PX * PY) / Be)))
    # base-e reference (enum3 leader)
    tx = D(am1 * D(ln64(x)))
    ty = R64(bm1 * ln64(1 - x))
    put("ref EXP(e(tx+e(ty-lnB)))", i, exp_v6(R64(tx + R64(ty - lnB))))

res = sorted(((sum(m[i] == rows[i]["excel"] for i in idx), k) for k, m in models.items()), reverse=True)
cases = sorted({rows[i]["case"] for i in idx})
print(want, len(idx))
for ex, k in res[:30]:
    m = models[k]
    per = " ".join(
        f"{c[-10:]}:{sum(m[i] == rows[i]['excel'] for i in idx if rows[i]['case'] == c)}/{sum(1 for i in idx if rows[i]['case'] == c)}"
        for c in cases
    )
    w4 = sum(abs(m[i] - rows[i]["excel"]) <= 4 * math.ulp(rows[i]["excel"]) for i in idx)
    print(f"{ex:4d} w4 {w4:4d} | {k} | {per}")

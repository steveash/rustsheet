"""Full-density enumeration with the pair-test winners: px = EXP(fl((a-1)*LN(x))), py from 1-x held exactly
(log1p-like), times every B / lnB form and combine order. Scores per region (exact / w4 / w15).
"""

import itertools
import math
import pickle
import sys

import mpmath

from lib import D, R64, exp_v6, exp_v6_ext, fmt, ln64, load_pdf, mpf, score, xl

mpmath.mp.prec = 200
rows = load_pdf()
N = len(rows)
X = [r["x"] for r in rows]
A = [r["a"] for r in rows]
Bv = [r["b"] for r in rows]
AB = [a + b for a, b in zip(A, Bv)]
LNx = xl("LN", X)
GLa, GLb, GLab = xl("GAMMALN", A), xl("GAMMALN", Bv), xl("GAMMALN", AB)
Ga, Gb, Gab = xl("GAMMA", A), xl("GAMMA", Bv), xl("GAMMA", AB)


def region(i):
    a, b = A[i], Bv[i]
    ra = "a<1" if a < 1 else ("a<8" if a < 8 else "a>=8")
    rb = "b<1" if b < 1 else ("b<=8" if b <= 8 else "b>8")
    return ra + "," + rb


REG = [region(i) for i in range(N)]
models = {}


def put(name, i, v):
    models.setdefault(name, [math.nan] * N)[i] = v


for i in range(N):
    x, a, b = mpf(X[i]), mpf(A[i]), mpf(Bv[i])
    am1, bm1 = D(a - 1), D(b - 1)
    lx = mpf(LNx[i])
    ye = 1 - x
    tx = D(am1 * lx)
    ly64 = ln64(ye)
    pxs = {"px=EXP(d)": mpf(exp_v6(tx))}
    pys = {
        "py=EXP(d(bm1*LN64(1-x)))": mpf(exp_v6(D(bm1 * ly64))),
        "py=EXP(d(bm1*d(LN64(1-x))))": mpf(exp_v6(D(bm1 * D(ly64)))),
        "py=EXP(e(bm1*LN64))": mpf(exp_v6(R64(bm1 * ly64))),
        "py=EXPext(e(bm1*LN64))": exp_v6_ext(R64(bm1 * ly64)),
        "py=exact": ye ** (b - 1),
    }
    ga, gb, gab = mpf(GLa[i]), mpf(GLb[i]), mpf(GLab[i])
    lnbs = {
        "ga+(gb-gab)": D(ga + D(gb - gab)),
        "(ga+gb)-gab": D(D(ga + gb) - gab),
        "gb+(ga-gab)": D(gb + D(ga - gab)),
        "e:ga+(gb-gab)": R64(ga + R64(gb - gab)),
    }
    Bs = {f"B=EXP({k})": mpf(exp_v6(v)) for k, v in lnbs.items()}
    Bs["B=EXPext(ga+(gb-gab))"] = exp_v6_ext(R64(ga + R64(gb - gab)))
    if all(math.isfinite(v) and v > 0 for v in (Ga[i], Gb[i], Gab[i])):
        G1, G2, G3 = mpf(Ga[i]), mpf(Gb[i]), mpf(Gab[i])
        Bs["B=Ga*Gb/Gab"] = mpf(D(D(G1 * G2) / G3))
        Bs["B=Ga*(Gb/Gab)"] = mpf(D(G1 * D(G2 / G3)))
        Bs["1/B=Gab/(Ga*Gb)"] = 1 / mpf(D(G3 / D(G1 * G2)))
    Bs["B=exact"] = mpmath.beta(a, b)
    Bs["B=exact_d"] = mpf(float(mpmath.beta(a, b)))
    for (kx, vx), (ky, vy), (kb, vb) in itertools.product(pxs.items(), pys.items(), Bs.items()):
        put(f"{kx} {ky} {kb} (px*py)/B", i, D(D(vx * vy) / vb))
        put(f"{kx} {ky} {kb} px*(py/B)", i, D(vx * D(vy / vb)))
        put(f"{kx} {ky} {kb} (px/B)*py", i, D(D(vx / vb) * vy))
        put(f"{kx} {ky} {kb} e(px*py)/B", i, D(R64(vx * vy) / vb))
        put(f"{kx} {ky} {kb} (py/B)*px", i, D(D(vy / vb) * vx))
    # single exp of the sum
    ty = D(bm1 * ly64)
    for kl, lb in lnbs.items():
        put(f"EXP((tx+ty)-lnB) {kl}", i, exp_v6(D(D(tx + ty) - lb)))
        put(f"EXP(tx+(ty-lnB)) {kl}", i, exp_v6(D(tx + D(ty - lb))))
        put(f"EXP((tx-lnB)+ty) {kl}", i, exp_v6(D(D(tx - lb) + ty)))
        put(f"EXP(tx-lnB)*py {kl}", i, D(mpf(exp_v6(D(tx - lb))) * pys["py=EXP(d(bm1*LN64(1-x)))"]))

pickle.dump(dict(models=models, reg=REG), open("enum2_models.pkl", "wb"))
regs = sorted(set(REG))
base = [r["exact"] for r in rows]


def line(name, m):
    per = "  ".join(
        f"{g}:{score(m, rows, [j for j in range(N) if REG[j] == g])[1]}/{REG.count(g)}" for g in regs
    )
    return f"{name[:88]:88s} {fmt(score(m, rows))} | {per}"


print(line("exact rounded", base))
res = sorted(models.items(), key=lambda kv: -score(kv[1], rows)[1])
for name, m in res[:40]:
    print(line(name, m))
print("== best per region")
for g in regs:
    idx = [j for j in range(N) if REG[j] == g]
    best = sorted(models.items(), key=lambda kv: -score(kv[1], rows, idx)[1])[:3]
    for name, m in best:
        print(g, fmt(score(m, rows, idx)), name)

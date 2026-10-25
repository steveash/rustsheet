"""Enumerate density compositions: power terms x power forms x Beta forms x combine orders.

Prints the score of every combination overall and per region; writes enum1_models.pkl
(name -> list of model doubles) for later region maps.
"""

import itertools
import math
import pickle
import sys

import mpmath

from lib import D, R64, exp_v6, exp_v6_ext, fmt, ln64, load_pdf, mpf, score, xl

rows = load_pdf()
N = len(rows)
X = [r["x"] for r in rows]
A = [r["a"] for r in rows]
B = [r["b"] for r in rows]
Y = [1.0 - x for x in X]
AB = [a + b for a, b in zip(A, B)]
LNx = xl("LN", X)
LNy = xl("LN", Y)
GLa = xl("GAMMALN", A)
GLb = xl("GAMMALN", B)
GLab = xl("GAMMALN", AB)
Ga = xl("GAMMA", A)
Gb = xl("GAMMA", B)
Gab = xl("GAMMA", AB)

mpmath.mp.prec = 300


def region(i):
    a, b = A[i], B[i]
    ra = "a<1" if a < 1 else ("a<8" if a < 8 else "a>=8")
    rb = "b<1" if b < 1 else ("b<=8" if b <= 8 else "b>8")
    return ra + "," + rb


REG = [region(i) for i in range(N)]


# ---- Beta function forms: return B as a double (or None), and lnB (double)
def lnB_forms(i):
    a, b = A[i], B[i]
    ga, gb, gab = mpf(GLa[i]), mpf(GLb[i]), mpf(GLab[i])
    out = {
        "lnB:ga+(gb-gab)": D(ga + D(gb - gab)),
        "lnB:(ga+gb)-gab": D(D(ga + gb) - gab),
        "lnB:gb+(ga-gab)": D(gb + D(ga - gab)),
        "lnB:smallfirst": D(ga + D(gb - gab)) if a <= b else D(gb + D(ga - gab)),
        "lnB:exact": float(mpmath.log(mpmath.beta(mpf(a), mpf(b)))),
    }
    return out


def B_forms(i, lnbs):
    a, b = A[i], B[i]
    out = {k.replace("lnB:", "B:exp "): exp_v6(v) for k, v in lnbs.items() if k != "lnB:exact"}
    ga, gb, gab = mpf(Ga[i]), mpf(Gb[i]), mpf(Gab[i])
    if all(math.isfinite(v) and v > 0 for v in (Ga[i], Gb[i], Gab[i])):
        out["B:Ga*Gb/Gab"] = D(D(ga * gb) / gab)
        out["B:Ga*(Gb/Gab)"] = D(ga * D(gb / gab))
        out["B:Gb*(Ga/Gab)"] = D(gb * D(ga / gab))
        out["B:Ga/Gab*Gb"] = D(D(ga / gab) * gb)
    out["B:exact"] = float(mpmath.beta(mpf(a), mpf(b)))
    return out


def power_forms(i):
    """x^(a-1) and y^(b-1) forms, and the log terms."""
    x, y, a, b = X[i], Y[i], A[i], B[i]
    am1, bm1 = D(mpf(a) - 1), D(mpf(b) - 1)
    lx, ly = mpf(LNx[i]), mpf(LNy[i])
    tx, ty = D(am1 * lx), D(bm1 * ly)
    px = {
        "px:exp(d(am1*ln))": exp_v6(tx),
        "px:exp(e(am1*ln))": exp_v6(R64(am1 * lx)),
        "px:exp(a*ln)/x": D(mpf(exp_v6(D(mpf(a) * lx))) / x),
        "px:pow_cr": float(mpf(x) ** am1),
    }
    py = {
        "py:exp(d(bm1*ln))": exp_v6(ty),
        "py:exp(e(bm1*ln))": exp_v6(R64(bm1 * ly)),
        "py:exp(b*ln)/y": D(mpf(exp_v6(D(mpf(b) * ly))) / y),
        "py:pow_cr": float(mpf(y) ** bm1),
    }
    return px, py, tx, ty, lx, ly, am1, bm1


models = {}


def put(name, i, v):
    models.setdefault(name, [math.nan] * N)[i] = v


for i in range(N):
    lnbs = lnB_forms(i)
    bfs = B_forms(i, lnbs)
    px, py, tx, ty, lx, ly, am1, bm1 = power_forms(i)
    for (kx, vx), (ky, vy) in itertools.product(px.items(), py.items()):
        if kx.split(":")[1].split("(")[0] != ky.split(":")[1].split("(")[0] and not (
            "pow" in kx and "pow" in ky
        ):
            # keep matched families plus full cross for the first family only
            if not (kx.startswith("px:exp(d") or ky.startswith("py:exp(d")):
                continue
        vx_, vy_ = mpf(vx), mpf(vy)
        for kb, vb in bfs.items():
            vb_ = mpf(vb)
            put(f"{kx} {ky} {kb} (px*py)/B", i, D(D(vx_ * vy_) / vb_))
            put(f"{kx} {ky} {kb} px*(py/B)", i, D(vx_ * D(vy_ / vb_)))
            put(f"{kx} {ky} {kb} (px/B)*py", i, D(D(vx_ / vb_) * vy_))
            put(f"{kx} {ky} {kb} px*py*(1/B)", i, D(D(vx_ * vy_) * D(1 / vb_)))
    # single-EXP forms
    for kl, lb in lnbs.items():
        lb_ = mpf(lb)
        put(f"EXP((tx+ty)-lnB) {kl}", i, exp_v6(D(D(mpf(tx) + ty) - lb_)))
        put(f"EXP(tx+(ty-lnB)) {kl}", i, exp_v6(D(mpf(tx) + D(ty - lb_))))
        put(f"EXP((tx-lnB)+ty) {kl}", i, exp_v6(D(D(mpf(tx) - lb_) + ty)))
        put(f"EXP(ext tx+ty-lnB) {kl}", i, exp_v6(R64(R64(R64(am1 * lx) + R64(bm1 * ly)) - lb_)))
        put(f"EXP(-lnB)*... {kl}", i, D(D(mpf(exp_v6(D(mpf(tx) + ty))) / exp_v6(lb))))
    if i % 200 == 0:
        print(i, file=sys.stderr)

pickle.dump(dict(models=models, reg=REG), open("enum1_models.pkl", "wb"))

regs = sorted(set(REG))
base = [r["exact"] for r in rows]
print(
    "exact rounded".ljust(80),
    fmt(score(base, rows)),
    " | ",
    "  ".join(
        f"{g}:{score(base, rows, [i for i in range(N) if REG[i] == g])[1]}/{REG.count(g)}" for g in regs
    ),
)
res = sorted(models.items(), key=lambda kv: -score(kv[1], rows)[1])
for name, m in res[:60]:
    s = score(m, rows)
    per = "  ".join(f"{g}:{score(m, rows, [i for i in range(N) if REG[i] == g])[1]}" for g in regs)
    print(name.ljust(80), fmt(s), " | ", per)

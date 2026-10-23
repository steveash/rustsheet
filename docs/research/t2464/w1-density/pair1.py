"""B-free test of the power terms: for each (case, a, b) seen at two or more x, the implied 1/B
q = excel / (px * py) must agree across x when px, py are Excel's forms (up to the final roundings).

Reports, per power-form pair, the rms and the share of pairs with |dq| <= 1.5 x 2^-53 relative,
plus the same for a synthetic control (data made by a known form + exact B, rounded).
"""

import collections
import itertools
import math

import mpmath

from lib import D, R64, exp_v6, load_pdf, mpf, xl

mpmath.mp.prec = 200
rows = load_pdf()
N = len(rows)
X = [r["x"] for r in rows]
A = [r["a"] for r in rows]
Bv = [r["b"] for r in rows]
Y = [1.0 - x for x in X]
LNx = xl("LN", X)
LNy = xl("LN", Y)


def forms(i):
    x, y, a, b = X[i], Y[i], A[i], Bv[i]
    am1, bm1 = D(mpf(a) - 1), D(mpf(b) - 1)
    lx, ly = mpf(LNx[i]), mpf(LNy[i])
    X_, Y_ = mpf(x), mpf(y)
    px = {
        "exp(d(am1*LN))": mpf(exp_v6(D(am1 * lx))),
        "exp(e(am1*LN))": mpf(exp_v6(R64(am1 * lx))),
        "exp(am1*ln_exact)": mpf(exp_v6(D(am1 * mpmath.log(X_)))),
        "exp(a*LN)/x": mpf(exp_v6(D(mpf(a) * lx))) / X_,
        "exp(a*LN)/x stored": mpf(D(mpf(exp_v6(D(mpf(a) * lx))) / X_)),
        "pow_cr": mpf(float(X_**am1)),
        "exact": X_ ** (mpf(a) - 1),
    }
    py = {
        "exp(d(bm1*LN))": mpf(exp_v6(D(bm1 * ly))),
        "exp(e(bm1*LN))": mpf(exp_v6(R64(bm1 * ly))),
        "exp(bm1*log1p(-x))": mpf(exp_v6(D(bm1 * mpmath.log1p(-X_)))),
        "exp(b*LN)/y": mpf(exp_v6(D(mpf(b) * ly))) / Y_,
        "pow_cr": mpf(float(Y_**bm1)),
        "exact_y": Y_ ** (mpf(b) - 1),
        "exact_1-x": (1 - X_) ** (mpf(b) - 1),
    }
    return px, py


F = [forms(i) for i in range(N)]
groups = collections.defaultdict(list)
for i, r in enumerate(rows):
    groups[(r["case"], A[i], Bv[i])].append(i)
pairs = [g for g in groups.values() if len(g) >= 2]
print("groups with >=2 x:", len(pairs))


def evaluate(target):
    out = []
    for kx, ky in itertools.product(F[0][0], F[0][1]):
        d = []
        for g in pairs:
            q = [target[i] / (F[i][0][kx] * F[i][1][ky]) for i in g]
            for j in range(1, len(q)):
                d.append(float((q[j] - q[0]) / q[0]) * 2**53)
        rms = math.sqrt(sum(v * v for v in d) / len(d))
        good = sum(abs(v) <= 1.5 for v in d)
        med = sorted(abs(v) for v in d)[len(d) // 2]
        out.append((rms, kx, ky, good, len(d), med))
    out.sort()
    return out


excel = [mpf(r["excel"]) for r in rows]
print("== Excel data")
for rms, kx, ky, good, n, med in evaluate(excel)[:20]:
    print(f"{kx:22s} {ky:22s} rms {rms:8.2f} med {med:6.2f} |dq|<=1.5: {good}/{n}")

# control: data from exp(d(am1*LN)) * exp(d(bm1*LN)) / B_exact, stored
ctl = []
for i in range(N):
    Bx = mpmath.beta(mpf(A[i]), mpf(Bv[i]))
    ctl.append(mpf(D(mpf(D(F[i][0]["exp(d(am1*LN))"] * F[i][1]["exp(d(bm1*LN))"])) / Bx)))
print("== control (truth exp(d(am1*LN)) exp(d(bm1*LN)))")
for rms, kx, ky, good, n, med in evaluate(ctl)[:6]:
    print(f"{kx:22s} {ky:22s} rms {rms:8.2f} med {med:6.2f} |dq|<=1.5: {good}/{n}")

print("== per-case, selected pairs (median |dq|, share <=1.5)")
sel = [("exact", "exact_1-x"), ("exp(am1*ln_exact)", "exp(bm1*log1p(-x))"), ("exp(d(am1*LN))", "exp(d(bm1*LN))"),
       ("exp(d(am1*LN))", "exact_1-x"), ("exact", "exact_y"), ("exp(e(am1*LN))", "exact_1-x")]
cases = sorted({k[0] for k in groups})
for kx, ky in sel:
    line = []
    for c in cases:
        d = []
        for key, g in groups.items():
            if key[0] != c or len(g) < 2:
                continue
            q = [excel[i] / (F[i][0][kx] * F[i][1][ky]) for i in g]
            d += [float((q[j] - q[0]) / q[0]) * 2**53 for j in range(1, len(q))]
        if d:
            med = sorted(abs(v) for v in d)[len(d) // 2]
            line.append(f"{c[-14:]}:{med:.2f},{sum(abs(v)<=1.5 for v in d)}/{len(d)}")
    print(kx, "|", ky, "\n   ", "  ".join(line))

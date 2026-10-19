"""2-D generator-fingerprint solve (T-1941 W5): find [a,b] so that a minimax parent's x=0 invariants R0 and R1
both equal Excel's, then report the R2, R3 residuals. Damped Newton with backtracking.
usage: solve3.py TARGET M N WEIGHT a0 b0   (TARGET, WEIGHT as in solve2.py)"""

import mpmath as mp, sys
from remez2 import remez2
from taylor0 import taylor

EX = [mp.mpf("-5.40585248391"), mp.mpf("134.942282379"), mp.mpf("-5937.32019206"), mp.mpf("281011.5087")]
tgt, m, n, wn = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
a, b = mp.mpf(sys.argv[5]), mp.mpf(sys.argv[6])
mp.mp.dps = 40


def A(x):
    return mp.loggamma(x) / (x - 1) if x != 1 else -mp.euler


def g(x):
    return A(x) / (x - 2) if x != 2 else 1 - mp.euler


f = A if tgt == "A" else g
W = {
    "rel": None,
    "absA": (lambda x: mp.mpf(1)) if tgt == "A" else (lambda x: 1 / abs(x - 2)),
    "absg": (lambda x: abs(x - 2)) if tgt == "A" else (lambda x: mp.mpf(1)),
}
ref = [None]


def inv(a, b):
    r = ref[0]
    if r is not None:
        a0, b0, X = r
        r = [a + (x - a0) * (b - a) / (b0 - a0) for x in X]
    try:
        p, q, E, X = remez2(f, a, b, m, n, w=W[wn], dps=40, grid=500, ref=r, tol="1e-12")
    except Exception:
        p, q, E, X = remez2(f, a, b, m, n, w=W[wn], dps=40, grid=500, tol="1e-12")
    ref[0] = (a, b, X)
    if tgt == "g":
        p = [-2 * p[0]] + [p[k - 1] - 2 * p[k] for k in range(1, len(p))] + [p[-1]]
    t = taylor(p, q)
    return [t[k] / EX[k] - 1 for k in range(4)], E


h = mp.mpf("1e-5")
r, E = inv(a, b)
for it in range(15):
    print(
        it,
        mp.nstr(a, 14),
        mp.nstr(b, 14),
        "E",
        mp.nstr(E, 4),
        "rel dR",
        [mp.nstr(v, 4) for v in r],
        flush=True,
    )
    if abs(r[0]) < 1e-12 and abs(r[1]) < 1e-11:
        break
    ra, _ = inv(a + h, b)
    rb, _ = inv(a, b + h)
    J = mp.matrix([[(ra[0] - r[0]) / h, (rb[0] - r[0]) / h], [(ra[1] - r[1]) / h, (rb[1] - r[1]) / h]])
    d = mp.lu_solve(J, mp.matrix([-r[0], -r[1]]))
    lam = mp.mpf(1)
    nr0 = abs(r[0]) + abs(r[1])
    while lam > 1e-4:
        na, nb = a + lam * d[0], b + lam * d[1]
        if mp.mpf("0.3") < na < nb - mp.mpf("0.05") and nb < mp.mpf("3"):
            try:
                rn, En = inv(na, nb)
                if abs(rn[0]) + abs(rn[1]) < nr0:
                    a, b, r, E = na, nb, rn, En
                    break
            except Exception:
                pass
        lam /= 2
    else:
        print("line search failed")
        break

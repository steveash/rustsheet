"""Generator-fingerprint solve (T-1941 W5).

Excel's rational A = N/D has x=0 Taylor invariants R0..R3 (pinned by c5_f99t, whose coordinates the near-1 data
pin to ~1e-10). They are invariant under any common factor F of N and D, so a parent N*/D* with Excel = round(N*F,
D*F) must reproduce them. For a weighted minimax family, solve the interval [a,b] so that R0 and R1 match, then
report the R2 and R3 mismatch as the test.

usage: solve2.py TARGET M N WEIGHT a0 b0 [fix]
  TARGET A: A = lgamma(x)/(x-1), parent degree (M,N), Excel N = parent numerator
         g: g = lgamma(x)/((x-1)(x-2)), Excel N = (x-2)*parent numerator (root at 2 exact)
  WEIGHT rel | absA | absg | absB (absolute in lgamma)
  fix    a or b holds that end and solves R0 only.
"""

import mpmath as mp, sys
from remez2 import remez2
from taylor0 import taylor

EX = [
    mp.mpf("-5.4058524839103"),
    mp.mpf("134.94228237913"),
    mp.mpf("-5937.3201920600"),
    mp.mpf("281011.50865"),
]
tgt, m, n, wn, a, b = (
    sys.argv[1],
    int(sys.argv[2]),
    int(sys.argv[3]),
    sys.argv[4],
    mp.mpf(sys.argv[5]),
    mp.mpf(sys.argv[6]),
)
fix = sys.argv[7] if len(sys.argv) > 7 else ""
mp.mp.dps = 40


def A(x):
    return mp.loggamma(x) / (x - 1) if x != 1 else -mp.euler


def g(x):
    return A(x) / (x - 2) if x != 2 else 1 - mp.euler  # removable: psi(2) = 1 - gamma


f = A if tgt == "A" else g
big = mp.mpf(10) ** 30
W = {
    "rel": None,
    "absA": (lambda x: mp.mpf(1)) if tgt == "A" else (lambda x: 1 / abs(x - 2)),
    "absg": (lambda x: abs(x - 2)) if tgt == "A" else (lambda x: mp.mpf(1)),
    "absB": (lambda x: 1 / abs(x - 1) if x != 1 else big)
    if tgt == "A"
    else (lambda x: 1 / abs((x - 1) * (x - 2)) if x != 1 else big),
}
ref = [None]


def inv(a, b):
    r = ref[0]
    if r is not None:
        a0, b0, X = r
        r = [a + (x - a0) * (b - a) / (b0 - a0) for x in X]
    try:
        p, q, E, X = remez2(f, a, b, m, n, w=W[wn], dps=40, grid=500, ref=r)
    except Exception:
        p, q, E, X = remez2(f, a, b, m, n, w=W[wn], dps=40, grid=500)
    ref[0] = (a, b, X)
    if tgt == "g":
        p = [-2 * p[0]] + [p[k - 1] - 2 * p[k] for k in range(1, len(p))] + [p[-1]]
    return taylor(p, q), E


h = mp.mpf("1e-4")
for it in range(10):
    t, E = inv(a, b)
    r0 = t[0] - EX[0]
    r1 = t[1] - EX[1]
    print(
        it,
        mp.nstr(a, 12),
        mp.nstr(b, 12),
        "E",
        mp.nstr(E, 4),
        "rel dR",
        [mp.nstr(t[k] / EX[k] - 1, 4) for k in range(4)],
        flush=True,
    )
    if abs(r0 / EX[0]) < 1e-11 and (fix or abs(r1 / EX[1]) < 1e-11):
        break
    if fix == "b":
        ta, _ = inv(a + h, b)
        d = [-r0 / ((ta[0] - t[0]) / h), 0]
    elif fix == "a":
        tb, _ = inv(a, b + h)
        d = [0, -r0 / ((tb[0] - t[0]) / h)]
    else:
        ta, _ = inv(a + h, b)
        tb, _ = inv(a, b + h)
        J = mp.matrix([[(ta[0] - t[0]) / h, (tb[0] - t[0]) / h], [(ta[1] - t[1]) / h, (tb[1] - t[1]) / h]])
        d = mp.lu_solve(J, mp.matrix([-r0, -r1]))
    s = min(1, mp.mpf("0.05") / max(abs(d[0]), abs(d[1]), mp.mpf("1e-30")))
    a = min(max(a + s * d[0], mp.mpf("0.3")), mp.mpf("0.98"))
    b = min(max(b + s * d[1], mp.mpf("1.05")), mp.mpf("2.6"))
    ref[0] = None if s < 1 else ref[0]

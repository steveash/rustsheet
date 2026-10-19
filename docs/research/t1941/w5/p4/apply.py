"""Apply the plateau bit readings to c5 and restore the function (T-1941 W5 part 4).
1. c5 + bit moves (mid-points of the nearest consistent ranges from stepreader.py real) + N2 +1 (tie reading).
2. Function restoration: least-squares integer moves of the coefficients that carry no plateau reading, so that the
   rational matches TARGET (c5's function, or c5's function plus the measured smooth bias) on [0.7, 1.5].
Prints exact far/near and the staircase chi^2 (plateau.py objective, all points)."""

import numpy as np, sys, os, json

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.argv = sys.argv[:1] + ["none"]
exec(open(os.path.join(HERE, "plateau.py")).read().split("if __name__")[0])
from sc import score
from metrics import XF, BF, XN, BN

MOVES = {("N", 1): -21, ("D", 3): -1, ("D", 2): 6, ("D", 1): 310, ("N", 0): 440, ("N", 2): 1}


def moved(spec):
    N = N5.copy()
    D = D5.copy()
    for (ch, k), j in spec.items():
        C = N if ch == "N" else D
        C[k] += j * np.spacing(abs(C[k]))
    return N, D


tmu, tse = runmean(b)


def report(tag, N, D):
    print(
        "%-28s far %.4f near %.4f | staircase chi2 %.0f"
        % (tag, score(XF, BF, N, D), score(XN, BN, N, D), chi2(tmu, tse, N, D)[0]),
        flush=True,
    )


report("c5", N5, D5)
N1, D1 = moved(MOVES)
report("c5 + plateau bits", N1, D1)
# function restoration with free = coefficients without a reading
FREE = [("N", k) for k in range(3, 10)] + [("D", k) for k in range(4, 10)]
xs = np.linspace(0.7, 1.5, 120).astype(np.longdouble)


def ratl(N, D):
    return np.polyval(N.astype(np.longdouble)[::-1], xs) / np.polyval(D.astype(np.longdouble)[::-1], xs)


target = ratl(N5, D5)
for it in range(3):
    cur = ratl(N1, D1)
    Nv = np.polyval(N1.astype(np.longdouble)[::-1], xs)
    Dv = np.polyval(D1.astype(np.longdouble)[::-1], xs)
    cols = []
    for ch, k in FREE:
        C = N1 if ch == "N" else D1
        u = np.spacing(abs(C[k]))
        cols.append((u * xs**k / Nv if ch == "N" else -u * xs**k / Dv).astype(float))
    M = np.array(cols).T
    rhs = ((target / cur) - 1).astype(float)
    sol, *_ = np.linalg.lstsq(M / 1e-17, rhs / 1e-17, rcond=None)
    for (ch, k), v in zip(FREE, sol):
        C = N1 if ch == "N" else D1
        C[k] += round(v) * np.spacing(abs(C[k]))
    print(
        "  restore it %d: max rel function diff vs c5 %.1e"
        % (it, float(np.abs(ratl(N1, D1) / target - 1).max()))
    )
report("c5 + bits + restored function", N1, D1)
open(os.path.join(HERE, "k3_bits_restored.txt"), "w").write(
    " ".join(v.hex() for v in N1) + "\n" + " ".join(v.hex() for v in D1) + "\n"
)

"""K3 top-down conditional solve of BOTH chains with all-row alias freedom (T-2122 run 4, HANDOFF2 15.2 step 3).
Stage j's slope equations (from w5/p5/ladder.py's log) are solved for the moves ks with the moves already found
above it held fixed. Each solve is JOINTALL's search (run 3): RANSAC over minimal subsets x alias shifts {-1,0,+1},
refine (LS -> nearest alias per row, kept within TOL and |alias| <= MAXALIAS), rank by rows kept, then total |alias|,
then rms. Order: N from HEAD (dN7, dN8, dN9 in ulps, given) then N5 -> dN6, N3 -> dN4+dN5; D6 -> dD7..dD9,
D5 -> dD6, D4 -> dD5. Prints every move in ulps and writes the moved set (no refit) to OUT.
usage: k3_lower.py LOG BASE OUT dN7 dN8 dN9   env PLAN, TOL (3e6), MAXALIAS (2), MINRUNG (10)"""

import sys, re, math, os, itertools
import numpy as np


def ld(f):
    l = open(f).read().split("\n")
    return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]


N0, D0 = ld(sys.argv[2])
BASE = {"N": N0, "D": D0}
eq = []  # (chain, j, x, slope, period)
a = None
for line in open(sys.argv[1]):
    m = re.match(r"anchor (\w) \(x ([\d.]+)\)", line)
    if m:
        a = float(m.group(2))
        continue
    m = re.match(r"\s+([ND])(\d): slope\s+([+-][\d.]+) cycles/x from (\d+) rungs \(([\d,]+)\)", line)
    if m and a is not None and int(m.group(4)) >= int(os.environ.get("MINRUNG", 10)):
        eq.append(
            (
                m.group(1),
                int(m.group(2)),
                a,
                float(m.group(3)),
                2.0 ** max(int(r) for r in m.group(5).split(",")),
            )
        )
TOL = float(os.environ.get("TOL", 3e6))
MAXA = int(os.environ.get("MAXALIAS", 2))


def ulp(C, k):
    return float(np.spacing(abs(C[k])))


def sval(C, j, x):
    return sum(float(C[k]) * x ** (k - j) for k in range(j, len(C)))


def drow(j, x, k):
    return (k - j) * x ** (k - j - 1) if k > j else 0.0  # d/dx of x^(k-j)


def fit(ch, js, ks, known):
    """Moves (ulps) for ks from the equations of stages js jointly, `known` (k -> ulps) applied first.
    Returns (moves, report)."""
    C = BASE[ch]
    rows = [(x, s, per, jj) for c_, jj, x, s, per in eq if c_ == ch and jj in js]
    name = "+".join("%s%d" % (ch, j) for j in js)
    if len(rows) < len(ks):
        return None, "stages %s: %d rows for %d unknowns" % (name, len(rows), len(ks))
    A = []
    y = []
    P = []
    for x, s, per, j in rows:
        u = math.ulp(abs(sval(C, j, x)))  # slopes are in cycles of ulp(S_j) per unit x
        A.append([drow(j, x, k) * ulp(C, k) / u for k in ks])
        y.append(s - sum(drow(j, x, k) * ulp(C, k) * v / u for k, v in known.items()))
        P.append(per)
    A, y, P = map(np.array, (A, y, P))
    n = len(ks)

    def refine(sol):
        for _ in range(30):
            r = A @ sol - y
            m = np.round(r / P)
            keep = (np.abs(r - m * P) < TOL) & (np.abs(m) <= MAXA)
            if keep.sum() < n:
                break
            new = np.linalg.lstsq(A[keep], y[keep] + m[keep] * P[keep], rcond=None)[0]
            if np.allclose(new, sol, rtol=0, atol=1e-6):
                break
            sol = new
        r = A @ sol - y
        m = np.round(r / P)
        res = r - m * P
        return sol, m, res, (np.abs(res) < TOL) & (np.abs(m) <= MAXA)

    sols = {}
    for sub in itertools.combinations(range(len(y)), n):
        sub = list(sub)
        for sh in itertools.product([-1, 0, 1], repeat=n):
            try:
                sol0 = np.linalg.solve(A[sub], y[sub] + np.array(sh) * P[sub])
            except np.linalg.LinAlgError:
                continue
            sol, m, res, keep = refine(sol0)
            if keep.sum() < n:
                continue
            key = (tuple(keep), tuple(m[keep].astype(int)))
            sc = float(np.sqrt(np.mean(res[keep] ** 2)))
            if key not in sols or sc < sols[key][0]:
                sols[key] = (sc, sol, m, res, keep)
    sc, sol, m, res, keep = sorted(
        sols.values(), key=lambda t: (-int(t[4].sum()), int(np.abs(t[2][t[4]]).sum()), t[0])
    )[0]
    rep = [
        "stages %s: %d/%d rows kept, rms %.3g cycles/x, moves %s"
        % (name, keep.sum(), len(y), sc, " ".join("%s%d %+.4g" % (ch, k, v) for k, v in zip(ks, sol)))
    ]
    rep += [
        "   %s%d x %.4f alias %+d residual %+12.0f %s" % (ch, r[3], r[0], mm, rr, "" if kk else "DROPPED")
        for r, mm, rr, kk in zip(rows, m, res, keep)
    ]
    return dict(zip(ks, sol)), "\n".join(rep)


moves = {"N": dict(zip((7, 8, 9), map(float, sys.argv[4:7]))), "D": {9: float(os.environ.get("DD9", 0))}}  # D9 is ill-determined (control)
# PLAN: ';'-separated steps CH:stages:moves; the default is the top-down order, N:3,5:4,5,6 fits N3+N5 rows jointly.
for step in os.environ.get("PLAN", "N:5:6;N:3:4,5;D:6:7,8,9;D:5:6;D:4:5").split(";"):
    ch, js, ks = step.split(":")
    mv, rep = fit(ch, [int(v) for v in js.split(",")], [int(v) for v in ks.split(",")], moves[ch])
    print(rep)
    if mv:
        moves[ch].update(mv)
for ch in "ND":
    print(
        "%s moves (ulps): %s"
        % (ch, " ".join("%s%d %+.4g" % (ch, k, v) for k, v in sorted(moves[ch].items())))
    )
N, D = N0.copy(), D0.copy()
for ch, C in (("N", N), ("D", D)):
    for k, v in moves[ch].items():
        C[k] += round(v) * np.spacing(abs(C[k]))
open(sys.argv[3], "w").write(" ".join(v.hex() for v in N) + "\n" + " ".join(v.hex() for v in D) + "\n")

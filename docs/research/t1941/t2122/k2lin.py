# T-2122 K2: per micro-ladder, fit a linear pre-rounding offset law at N.s3, off(x) = phi + s*(x - x_L) in ulp(s3),
# on cand9 stored, by exact rate (coarse grid then refine). Prints (s, phi, rate) per ladder plus the rate at off = 0.
# Planted control: CONTROL=s,phi synthesises Excel's column from cand9 with that law first (must recover it).
# SITE=s3 (default) | p4 | s2 ... picks the N op.
# usage (repo root): python3 docs/research/t1941/t2122/k2lin.py [LADDERS]
import sys, glob, os, numpy as np

sys.path.insert(0, "docs/research/t1941/review3")
from spec import load

LD = np.longdouble


def sd(v):
    return v.astype(np.float64).astype(LD)


def ulp(v):
    a = np.abs(v.astype(np.float64))
    return (np.nextafter(a, np.inf) - a).astype(LD)


PQ = np.load("SCRATCH/cand9_PQ.npy")
P = [float(v) for v in PQ[0]]
Q = [float(v) for v in PQ[1][:9]]


SITE = os.environ.get("SITE", "s3")  # N op carrying the offset: s<j> (sum) or p<j> (product)
SK, SJ = SITE[0], int(SITE[1:])


def nchain(x, off):
    s = LD(P[0]) + 0 * x
    for i in range(1, len(P)):
        pr = s * x
        if SK == "p" and i == SJ:
            pr = pr + off * ulp(sd(pr))
        v = sd(pr) + LD(P[i])
        if SK == "s" and i == SJ:
            v = v + off * ulp(sd(v))
        s = sd(v)
    return s


def dchain(x):
    s = LD(Q[0]) + 0 * x
    for i in range(1, len(Q)):
        s = sd(sd(s * x) + LD(Q[i]))
    return s


def ladder(tag):
    X = []
    Y = []
    for f in sorted(glob.glob(f"eval/suites/probes-numeric/cases/kmyst-gam-k2m{tag}*.oracle.json")):
        x, y = load(os.path.basename(f)[len("kmyst-gam-") : -len(".oracle.json")])
        X.append(x)
        Y.append(y)
    X = np.concatenate(X)
    Y = np.concatenate(Y)
    o = np.argsort(X)
    return X[o], Y[o]


def fit(X, Y):
    Xl = X.astype(LD)
    t = sd(Xl - LD(2))
    D = dchain(Xl)
    dx = Xl - LD(X[0])

    def rate(s, phi):
        return np.mean(sd(t * sd(nchain(Xl, LD(phi) + LD(s) * dx) / D)).astype(np.float64) == Y)

    best = (rate(0, 0), 0.0, 0.0)
    for s in np.arange(-2000, 2001, 20):
        for phi in np.arange(0, 1, 1 / 16):
            r = rate(s, phi)
            if r > best[0]:
                best = (r, s, phi)
    r, s0, p0 = best
    for s in np.arange(s0 - 20, s0 + 21, 2):
        for phi in np.arange(p0 - 1 / 16, p0 + 1 / 16 + 1e-9, 1 / 64):
            r = rate(s, phi)
            if r > best[0]:
                best = (r, s, phi)
    return rate(0, 0), best


LAD = sys.argv[1] if len(sys.argv) > 1 else "zabcde"
ctl = os.environ.get("CONTROL")
for tag in LAD:
    X, Y = ladder(tag)
    sub = (
        np.arange(len(X)) % 3 == 0
    )  # every third point keeps the grid search affordable; runs stay intact in x
    X = X[sub]
    Y = Y[sub]
    if ctl:
        cs, cp = (float(v) for v in ctl.split(","))
        Xl = X.astype(LD)
        Y = sd(sd(Xl - LD(2)) * sd(nchain(Xl, LD(cp) + LD(cs) * (Xl - LD(X[0]))) / dchain(Xl))).astype(
            np.float64
        )
    r0, (r, s, phi) = fit(X, Y)
    print(
        f"ladder {tag} x0 {X[0]:.5f}: off=0 {r0:.4f} -> linear law s {s:+.0f} cycles/x phi {phi % 1:.4f}: {r:.4f}",
        flush=True,
    )

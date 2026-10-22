# T-2122 K2: which op carries the per-window offset? For pooled windows (W runs of 256) on the micro-ladders, the best
# constant pre-rounding offset (1/32-ulp steps) at every N and D product (p) and sum (s) site of cand9 stored, and its
# exact-rate gain. A winding offset at the true site reads as a clean ramp with unit wraps; proxies read distorted.
# usage (repo root): python3 docs/research/t1941/t2122/k2stagewin.py [W] [LADDERS]
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


def chain(c, x, kind=None, j=-1, off=0.0):
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        pr = s * x
        if kind == "p" and i == j:
            pr = pr + off * ulp(sd(pr))
        v = sd(pr) + LD(c[i])
        if kind == "s" and i == j:
            v = v + off * ulp(sd(v))
        s = sd(v)
    return s


W = int(sys.argv[1]) if len(sys.argv) > 1 else 5
LAD = sys.argv[2] if len(sys.argv) > 2 else "zabcde"
offs = np.arange(-16, 16) / 32
SITES = [("N", k, j) for j in range(1, 10) for k in "ps"] + [("D", k, j) for j in range(1, 9) for k in "ps"]
for tag in LAD:
    X = []
    Y = []
    for f in sorted(glob.glob(f"eval/suites/probes-numeric/cases/kmyst-gam-k2m{tag}*.oracle.json")):
        x, y = load(os.path.basename(f)[len("kmyst-gam-") : -len(".oracle.json")])
        X.append(x)
        Y.append(y)
    X = np.concatenate(X)
    Y = np.concatenate(Y)
    o = np.argsort(X)
    X = X[o]
    Y = Y[o]
    n = len(X) // 256
    res = {s: [] for s in SITES}
    xs = []
    for w in range(0, n, W):
        Xl = X[w * 256 : (w + W) * 256].astype(LD)
        Yb = Y[w * 256 : (w + W) * 256]
        t = sd(Xl - LD(2))
        xs.append(float(Xl[0]))
        N0 = chain(P, Xl)
        D0 = chain(Q, Xl)
        base = np.mean(sd(t * sd(N0 / D0)).astype(np.float64) == Yb)
        for s in SITES:
            ch, k, j = s
            if ch == "N":
                sc = [
                    np.mean(sd(t * sd(chain(P, Xl, k, j, of) / D0)).astype(np.float64) == Yb) for of in offs
                ]
            else:
                sc = [
                    np.mean(sd(t * sd(N0 / chain(Q, Xl, k, j, of))).astype(np.float64) == Yb) for of in offs
                ]
            b = int(np.argmax(sc))
            res[s].append((offs[b], sc[b] - base))
    print(f"ladder {tag}: {len(xs)} windows from {xs[0]:.5f} step {xs[1] - xs[0]:.1e}")
    rank = sorted(SITES, key=lambda s: -np.mean([g for _, g in res[s]]))
    for s in rank[:8]:
        r = res[s]
        d = np.diff([o for o, _ in r])
        d = (d + 0.5) % 1 - 0.5
        print(
            f"   {s[0]}.{s[1]}{s[2]}: mean gain {np.mean([g for _, g in r]):+.4f}  step coherence {abs(np.mean(np.exp(2j * np.pi * d))):.3f}  "
            + " ".join(f"{o:+.2f}" for o, _ in r)
        )

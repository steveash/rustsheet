# T-2122 K2: c3(x) along the micro-ladders read on pooled windows (W consecutive 256-point runs), offsets in 1/32
# ulp(s3) steps including 0, so regions where c3 ~ 0 are visible (c3all.py's "informative" filter hides them).
# Set: cand9 stored (SCRATCH/cand9_PQ.npy). usage (repo root): python3 docs/research/t1941/t2122/k2c3window.py [W]
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


def chain_off(c, x, stage, off):
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        v = sd(s * x) + LD(c[i])
        if i == stage:
            v = v + off * ulp(sd(v))
        s = sd(v)
    return s


W = int(sys.argv[1]) if len(sys.argv) > 1 else 5
offs = np.arange(-16, 16) / 32
for tag in "zabcde":
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
    out = []
    for w in range(0, n, W):
        Xl = X[w * 256 : (w + W) * 256].astype(LD)
        Yb = Y[w * 256 : (w + W) * 256]
        t = sd(Xl - LD(2))
        D = chain_off(Q, Xl, 0, 0)
        prof = np.array(
            [np.mean(sd(t * sd(chain_off(P, Xl, 3, of) / D)).astype(np.float64) == Yb) for of in offs]
        )
        k = int(np.argmax(prof))
        out.append((float(Xl[0]), offs[k], prof[16], prof[k]))
    s0 = np.mean([r[2] for r in out])
    s1 = np.mean([r[3] for r in out])
    print(f"ladder {tag}: {len(out)} windows of {W} runs; score at c3=0 {s0:.4f} -> per-window c3 {s1:.4f}")
    print("   " + " ".join(f"{r[0]:.5f}:{r[1]:+.3f}({r[3] - r[2]:+.3f})" for r in out))

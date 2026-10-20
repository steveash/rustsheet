# T-1941 W6: reviewer 3's K2 hand-off term, tested on (1.5,2]: at stage k the sum is formed in extended with a hidden
# c/x, S_k = RN(P_k + c_k + c/x), and stage k-1 uses the stored coefficient fl(c_{k-1} - c), an algebraic no-op that moves
# the S_k rounding decisions. For each stage k of N and D, scan c = t * ulp(S_{k-1}) (t in [-T, T), step 1/Q), score exact
# R. Usage: hidscan.py RFILE CHAIN [npts] [T] [Q]   env STAGES (e.g. N3,N4,D5; default all)
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain, nrm

rf, chain = sys.argv[1], sys.argv[2]
npts = int(sys.argv[3]) if len(sys.argv) > 3 else 100000
T = float(sys.argv[4]) if len(sys.argv) > 4 else 1.0
Q = int(sys.argv[5]) if len(sys.argv) > 5 else 32
rows = [l.split() for l in open(rf)]
idx = np.random.default_rng(0).choice(len(rows), min(npts, len(rows)), replace=False)
x = np.array([float.fromhex(rows[i][0]) for i in idx])
lo = np.array([float.fromhex(rows[i][2]) for i in idx])
hi = np.array([float.fromhex(rows[i][3]) for i in idx])
del rows
N, D = read_chain(chain)
N = [v / nrm(D) for v in N]
D = [v / nrm(D) for v in D]
XL = x.astype(L)


def chainval(C, k=None, c=0.0):
    s = L(C[-1]) * np.ones_like(XL)
    for j in range(len(C) - 2, -1, -1):
        p = L(st(s * XL))
        cj = st(L(C[j]) - L(c)) if (k is not None and j == k - 1) else C[j]
        v = p + L(cj)
        if k is not None and j == k:
            v = v + L(c) / XL
        s = L(st(v))
    return s


def score(Nv, Dv):
    R = st(Nv / Dv)
    return np.mean((R >= lo) & (R <= hi))


Nb, Db = chainval(N), chainval(D)
base = score(Nb, Db)
print("base exact %.4f" % base, flush=True)
want = set(os.environ.get("STAGES", "").split(",")) - {""}
for ch, C in (("N", N), ("D", D)):
    for k in range(1, len(C) - 1):
        if want and "%s%d" % (ch, k) not in want:
            continue
        # the grid of S_{k-1} at the median x
        s = L(C[-1]) * np.ones_like(XL)
        for j in range(len(C) - 2, k - 2, -1):
            s = L(st(L(st(s * XL)) + L(C[j])))
        u = float(np.median(np.spacing(np.abs(st(s)))))
        best = (base, 0.0)
        for t in np.arange(-T, T, 1.0 / Q):
            c = t * u
            v = score(chainval(C, k, c), Db) if ch == "N" else score(Nb, chainval(C, k, c))
            if v > best[0]:
                best = (v, t)
        print("  %s stage %d: best t %+.4f (c = %.4g) exact %.4f (base %.4f)" % (ch, k, best[1], best[1] * u, best[0], base), flush=True)

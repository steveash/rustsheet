# T-1941 W2: joint multi-anchor early-stage search. args: in.pkl out.pkl anchors_csv rounds
import numpy as np, pickle, sys
from ysearch import Scorer, dsearch
p, q = pickle.load(open(sys.argv[1], "rb")); p = list(p); q = list(q)
X = np.load("allX.npy"); Y = np.load("allY.npy"); E = np.concatenate([np.load("e6X.npy"), np.load("e8X.npy")]); EY = np.concatenate([np.load("e6Y.npy"), np.load("e8Y.npy")])
rng = np.random.default_rng(0)
anchors = [float(a) for a in sys.argv[3].split(",")]
parts = []
for a in anchors:
    for A, B in [(E, EY), (X, Y)]:
        m = np.abs(A - a) < 1e-6
        if m.sum() >= 512:
            i = rng.choice(np.where(m)[0], min(3000, m.sum()), replace=False); parts.append((A[i], B[i]))
sc = Scorer(np.concatenate([a for a, b in parts]), np.concatenate([b for a, b in parts]))
f = sc
NT = sum(len(a) for a, b in parts)
def per():
    return " ".join("%.4f" % (Scorer(a, b)(p, q) / len(a)) for a, b in parts)
print("start", f(p, q) / NT, per(), flush=True)
for rnd in range(int(sys.argv[4])):
    for which, maxoff in [("p", 8), ("q", 7)]:
        for kk in (1, 2, 3, 4):
            for off in range(0, maxoff):
                if off + kk >= len(p if which == "p" else q): continue
                for T, st in [(3000, 10), (30, 1)]:
                    p, q, b = dsearch(p, q, f, which, kk, T, st, off)
    print(rnd, "%.4f" % (f(p, q) / NT), per(), flush=True)
    pickle.dump((p, q), open(sys.argv[2], "wb"))

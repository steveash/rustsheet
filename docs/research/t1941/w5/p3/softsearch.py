"""Exact-match search along soft directions (T-1941 W5 part 3; W4's G method).
Soft directions: SVD of the function-preservation map (relative N/D change per ulp move of each free coefficient,
80 points of [0.7,1.5]). Moves u = round(s * v_k) in ulps for the K smallest singular directions and spans
s = +-{0.3 .. 3e5}; greedy acceptance on far+near exact rate (subsample), several sweeps.
usage: softsearch.py SETFILE|c5 OUT [K] [SWEEPS]"""

import numpy as np, sys, os

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import candidate, score_batch, score
from metrics import XF, BF, XN, BN

src = sys.argv[1]
if src == "c5":
    N = candidate("N3").copy()
    D = candidate("D3").copy()
    D[3] = np.nextafter(D[3], -np.inf)
    N[2] = np.nextafter(N[2], np.inf)
else:
    l = open(src).read().split("\n")
    N = np.array([float.fromhex(v) for v in l[0].split()])
    D = np.array([float.fromhex(v) for v in l[1].split()])
K = int(sys.argv[3]) if len(sys.argv) > 3 else 16
SW = int(sys.argv[4]) if len(sys.argv) > 4 else 3
X = np.r_[XF[:25000], XN[:25000]]
B = np.r_[BF[:25000], BN[:25000]]
nf = 25000


def sc_many(P, Q):
    return score_batch(X[:nf], B[:nf], P, Q) + score_batch(X[nf:], B[nf:], P, Q)


xs = np.linspace(0.7, 1.5, 80).astype(np.longdouble)


def softdirs(N, D):
    Nv = np.polyval(N[::-1].astype(np.longdouble), xs)
    Dv = np.polyval(D[::-1].astype(np.longdouble), xs)
    cols = [np.spacing(abs(N[i])) * xs**i / Nv for i in range(len(N))] + [
        -np.spacing(abs(D[i])) * xs**i / Dv for i in range(1, len(D))
    ]
    M = np.array(cols, dtype=float).T
    U, s, Vt = np.linalg.svd(M / 1e-16, full_matrices=False)
    return s, Vt


def apply(N, D, u):
    N2 = N.copy()
    D2 = D.copy()
    for i in range(len(N)):
        N2[i] = N[i] + round(u[i]) * np.spacing(abs(N[i]))
    for k, i in enumerate(range(1, len(D))):
        D2[i] = D[i] + round(u[len(N) + k]) * np.spacing(abs(D[i]))
    return N2, D2


best = sc_many(N[None], D[None])[0]
print(
    "start far+near %.4f  (far %.4f near %.4f)"
    % (best / 2, score(X[:nf], B[:nf], N, D), score(X[nf:], B[nf:], N, D)),
    flush=True,
)
spans = [0.3, 1, 3, 10, 30, 100, 300, 1e3, 3e3, 1e4, 3e4, 1e5, 3e5]
for sw in range(SW):
    s, Vt = softdirs(N, D)
    for k in range(len(s) - 1, len(s) - 1 - K, -1):
        cands = [apply(N, D, sg * sp * Vt[k]) for sp in spans for sg in (1, -1)]
        P = np.array([c[0] for c in cands])
        Q = np.array([c[1] for c in cands])
        sc = sc_many(P, Q)
        i = int(np.argmax(sc))
        if sc[i] > best:
            best = sc[i]
            N, D = cands[i]
    print(" sweep %d: far+near %.4f" % (sw, best / 2), flush=True)
print(
    "final: far %.4f near %.4f | metrics-sample far %.4f near %.4f"
    % (score(X[:nf], B[:nf], N, D), score(X[nf:], B[nf:], N, D), score(XF, BF, N, D), score(XN, BN, N, D))
)
open(sys.argv[2], "w").write(" ".join(v.hex() for v in N) + "\n" + " ".join(v.hex() for v in D) + "\n")

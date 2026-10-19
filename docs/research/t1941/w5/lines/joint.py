"""Joint phase-lattice solve over all 19 free coefficients (T-1941 W5).

Rows: (a) per-run tie-line phases of D.S3 (x < 1) and N.S2, modulo 1; (b) function preservation,
(dN(x) - A(x) dD(x))/N(x) ~ 0 at 40 points of [0.7, 1.5] with tolerance FTOL. Unknowns: integer ulp moves of
N0..N9 and D1..D9 relative to a base set (c5_f99t with D3, N2 made even). CVP by fpylll.
usage: joint.py [control]        env: BOUND (default 1e6 ulps), FTOL (default 1e-16)
"""

import numpy as np, sys, os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res, score
from metrics import XF, BF, XN, BN
from fpylll import IntegerMatrix, CVP, LLL

control = len(sys.argv) > 1
N0 = PK.N.copy()
D0 = PK.D.copy()
D0[3] = np.nextafter(D0[3], -np.inf)
N0[2] = np.nextafter(N0[2], np.inf)
LINES = [(t[0], int(t[1]), float(t.split(":")[1]) if ":" in t else 9.0) for t in os.environ.get("LINES", "D3:1.0,N2:0.8").split(",")]  # (chain, stage, x max) tie lines
idxN = list(range(10))
idxD = list(range(1, 10))
U = np.r_[[ulp(N0[i]) for i in idxN], [ulp(D0[i]) for i in idxD]]


def apply(u):
    N = N0.copy()
    D = D0.copy()
    for k, i in enumerate(idxN):
        N[i] += u[k] * ulp(N0[i])
    for k, i in enumerate(idxD):
        D[i] += u[len(idxN) + k] * ulp(D0[i])
    return N, D


def line_runs(ch, J, xmax):
    C = N0 if ch == "N" else D0
    out = []
    for s in PK.segs:
        xm = PK.x[s][len(s) // 2]
        st, _, _ = stages(C, xm)
        P, S = [(P, S) for j, P, dP, S, dS in st if j == J][0]
        if np.frexp(S)[1] == np.frexp(P)[1] + 1 and xm < xmax:
            out.append((s, xm, 2 * ulp(S)))
    return out


def measure(B):
    rows = []
    ph = []
    for ch, J, xmax in LINES:
        C = N0 if ch == "N" else D0
        for s, xm, u2 in line_runs(ch, J, xmax):
            xx = PK.x[s]
            Sv = np.polyval(C.astype(np.longdouble)[J:][::-1], xx.astype(np.longdouble))
            E = np.exp(-2j * np.pi * np.mod(Sv / np.longdouble(u2), 1.0).astype(float)) * np.hanning(len(xx))
            d = PK.resid(xx, B[s])
            m = PK.resid(xx, res(xx, N0, D0))
            r = np.zeros(19)
            for i in range(J, 10):
                k = i if ch == "N" else 10 + idxD.index(i)
                r[k] = U[k] * xm ** (i - J) / u2
            rows.append(r)
            ph.append(np.angle(np.dot(d - d.mean(), E) / np.dot(m - m.mean(), E)) / 2 / np.pi)
    return np.array(rows), np.array(ph)


# function rows: relative change of N/D per ulp move
xs = np.linspace(0.7, 1.5, 40)
Nv = np.polyval(N0[::-1], xs)
Dv = np.polyval(D0[::-1], xs)
F = np.zeros((len(xs), 19))
for k, i in enumerate(idxN):
    F[:, k] = U[k] * xs**i / Nv
for k, i in enumerate(idxD):
    F[:, 10 + k] = -U[10 + k] * xs**i / Dv
FREE0 = os.environ.get("FREE", "N0-9,D1-9")
_fr = set()
for f in FREE0.split(","):
    lo, hi = f[1:].split("-"); _fr |= {(10 if f[0] == "D" else 0) + (i - 1 if f[0] == "D" else i) for i in range(int(lo), int(hi) + 1)}
BNDT = np.array([1e9 if k in _fr else 0 for k in range(19)])
Ball = np.zeros_like(PK.x)
allidx = np.concatenate(PK.segs)
truth = None
if control:
    # realistic truth: a function-preserving move (sloppy SVD directions of F), ~1e4..1e5 ulps
    _, sv, Vt = np.linalg.svd(F / np.abs(U) * np.abs(U), full_matrices=False)
    rng = np.random.default_rng(3)
    mv = sum(rng.normal() * Vt[k] for k in range(15, 19))
    truth = np.round(mv / np.abs(mv).max() * float(os.environ.get("TSIZE", 3e4))).astype(int)
    if os.environ.get("TRUTH") == "rand":
        fr = [k for k in range(19) if BNDT[k] > 1]
        truth = np.zeros(19, int); truth[fr] = rng.integers(-int(float(os.environ.get("TSIZE", 3e4))), int(float(os.environ.get("TSIZE", 3e4))), len(fr))
    Nt, Dt = apply(truth)
    Ball[allidx] = res(PK.x[allidx], Nt, Dt)
    print("truth function change max %.1e" % np.abs(F @ truth).max())
else:
    Ball[allidx] = PK.b[allidx]
Ph, ph = measure(Ball)
print("phase rows", len(ph))
BOUND = float(os.environ.get("BOUND", 1e6))
FREE = os.environ.get("FREE", "N0-9,D1-9")
free = set()
for f in FREE.split(","):
    lo, hi = f[1:].split("-"); free |= {(10 if f[0] == "D" else 0) + (i - 1 if f[0] == "D" else i) for i in range(int(lo), int(hi) + 1)}
BND = np.array([BOUND if k in free else 1e-3 for k in range(19)])
FTOL = float(os.environ.get("FTOL", 1e-16))
SC = 10**8
M = 40.0
Wf = M * 0.5 / FTOL  # FTOL relative <-> half a phase-cycle unit... (weights)
K = 19
R = len(ph)
rows = []
for k in range(K):
    rows.append(
        [int(round(SC / BND[k])) if kk == k else 0 for kk in range(K)]
        + [int(round(SC * M * Ph[r, k])) for r in range(R)]
        + [int(round(SC * Wf * F[s, k])) for s in range(len(xs))]
    )
for r in range(R):
    rows.append([0] * K + [int(round(SC * M)) if rr == r else 0 for rr in range(R)] + [0] * len(xs))
Bm = IntegerMatrix.from_matrix(rows)
LLL.reduction(Bm); from fpylll import BKZ; BKZ.reduction(Bm, BKZ.Param(block_size=int(os.environ.get("BKZ", 20))))
best = None
for sgn in (1, -1):
    tgt = [0] * K + [int(round(SC * M * sgn * p)) for p in ph] + [0] * len(xs)
    v = CVP.closest_vector(Bm, tgt, method=os.environ.get("CVPM", "fast"))
    u = np.array([round(v[k] * BND[k] / SC) for k in range(K)])
    pr = (sgn * ph - Ph @ u + 0.5) % 1 - 0.5
    print(
        "sign %+d: phase rms %.3f  func max %.1e  max|u| %d"
        % (sgn, pr.std(), np.abs(F @ u).max(), np.abs(u).max())
    )
    if truth is not None:
        print("   recovered == truth:", bool(np.all(u == truth)), " max|u-truth|", np.abs(u - truth).max())
    else:
        N1, D1 = apply(u)
        print(
            "   exact: base far %.4f near %.4f -> far %.4f near %.4f"
            % (score(XF, BF, N0, D0), score(XN, BN, N0, D0), score(XF, BF, N1, D1), score(XN, BN, N1, D1))
        )
        np.save("lines/joint_u_%+d.npy" % sgn, u)

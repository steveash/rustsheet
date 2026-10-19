"""Phase-lattice solve on one triangle (tie) line, T-1941 W5.

For a sum stage S_j = fl(P_j + c_j) one binade above P_j with c_j on P_j's grid, the round-half-even ties put a
line at f = S_j'*h/(2 ulp S_j). The per-run phase of data relative to an exact simulation of a base set is
    dphi_r = (S_j^Excel(x_r) - S_j^base(x_r)) / (2 ulp S_j)   (mod 1)
and S_j = sum_{i>=j} c_i x^(i-j) is linear in the integer ulp moves u_i of the coefficients c_j..c_top.
Solve the closest-vector problem for the u_i (fpylll), then verify by exact score.

usage: latsolve.py CHAIN J [control]
"""

import numpy as np, sys, os
from fractions import Fraction as Fr

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res, score
from metrics import XF, BF, XN, BN
from fpylll import IntegerMatrix, CVP, LLL

ch, J = sys.argv[1], int(sys.argv[2])
control = len(sys.argv) > 3
N0 = PK.N.copy()
D0 = PK.D.copy()
D0[3] = np.nextafter(D0[3], -np.inf)  # Excel's D3 is even (tie line present)
N0[2] = np.nextafter(N0[2], np.inf)  # Excel's N2 is even


def chainof(N, D):
    return N if ch == "N" else D


def runs_for(C):
    out = []
    for s in PK.segs:
        xx = PK.x[s]
        xm = xx[len(xx) // 2]
        st, _, _ = stages(C, xm)
        P, S = [(P, S) for j, P, dP, S, dS in st if j == J][0]
        if np.frexp(S)[1] == np.frexp(P)[1] + 1 and xm < float(os.environ.get("XMAX", 9)):
            out.append((s, xm, 2 * ulp(S)))
    return out


def phases(N, D, B):
    """per-run phase of target residual B (array over all points) relative to the base sim"""
    C = chainof(N, D)
    ph = []
    for s, xm, u2 in runs_for(C):
        xx = PK.x[s]
        XL = xx.astype(np.longdouble)
        Sv = np.polyval(C.astype(np.longdouble)[J:][::-1], XL)
        E = np.exp(-2j * np.pi * np.mod(Sv / np.longdouble(u2), 1.0).astype(float)) * np.hanning(len(xx))
        d = PK.resid(xx, B[s])
        m = PK.resid(xx, res(xx, N, D))
        Ad = np.dot(d - d.mean(), E)
        As = np.dot(m - m.mean(), E)
        ph.append((xm, u2, np.angle(Ad / As) / 2 / np.pi, abs(Ad), abs(As)))
    return np.array(ph)


C0 = chainof(N0, D0)
idx = list(range(J, len(C0)))
truth = None
Bt = PK.b
if control:  # synthetic Excel: base plus a known move on c_j..c_top
    rng = np.random.default_rng(7)
    truth = {i: int(rng.integers(-3000, 3000)) for i in idx}
    if J == 3 and ch == "D":
        truth[3] = 2 * int(rng.integers(-1500, 1500))  # keep D3 even
    Nt = N0.copy()
    Dt = D0.copy()
    Ct = chainof(Nt, Dt)
    for i, k in truth.items():
        Ct[i] = C0[i] + k * ulp(C0[i])
    Bt = np.zeros_like(PK.x)
    allidx = np.concatenate([s for s in PK.segs])
    Bt[allidx] = res(PK.x[allidx], Nt, Dt)
ph = phases(N0, D0, Bt)
print("runs", len(ph), "amp data %.1f sim %.1f" % (ph[:, 3].mean(), ph[:, 4].mean()))
# a_{r,i} = ulp(c_i) * x_r^(i-J) / (2 ulp S_j(x_r))
A = np.array([[ulp(C0[i]) * r[0] ** (i - J) / r[1] for i in idx] for r in ph])
bound = [4000.0] * len(idx) if control else [float(os.environ.get("BOUND", 1e6))] * len(idx)
SC = 10**6
M = 40.0
R = len(ph)
K = len(idx)
rows = []
for k in range(K):
    rows.append(
        [int(round(SC / bound[k])) if kk == k else 0 for kk in range(K)]
        + [int(round(SC * M * A[r, k])) for r in range(R)]
    )
for r in range(R):
    rows.append([0] * K + [int(round(SC * M)) if rr == r else 0 for rr in range(R)])
for sgn in (1, -1):
    Bm = IntegerMatrix.from_matrix(rows)
    LLL.reduction(Bm)
    tgt = [0] * K + [int(round(SC * M * sgn * p)) for p in ph[:, 2]]
    v = CVP.closest_vector(Bm, tgt)
    u = [round(v[k] * bound[k] / SC) for k in range(K)]
    resid = np.array([(sgn * p - A[r] @ u + 0.5) % 1 - 0.5 for r, p in enumerate(ph[:, 2])])
    print("sign %+d: u =" % sgn, dict(zip(idx, u)), " phase rms %.3f" % resid.std())
    if truth:
        print("   truth", truth)
    if not control:
        N1 = N0.copy()
        D1 = D0.copy()
        C1 = chainof(N1, D1)
        for i, k in zip(idx, u):
            C1[i] = C0[i] + k * ulp(C0[i])
        print(
            "   exact: base far %.4f near %.4f -> new far %.4f near %.4f"
            % (score(XF, BF, N0, D0), score(XN, BN, N0, D0), score(XF, BF, N1, D1), score(XN, BN, N1, D1))
        )
if truth:
    ut = np.array([truth[i] for i in idx], float)
    for sgn in (1, -1):
        rr = np.array([(sgn*p - A[r] @ ut + 0.5) % 1 - 0.5 for r, p in enumerate(ph[:, 2])])
        print('truth sign %+d phase resid rms %.3f' % (sgn, rr.std()), np.round(rr[:12], 2))
    print('pred phases', np.round(((A @ ut)+0.5) % 1 - 0.5, 2)[:12]); print('meas', np.round(ph[:12, 2], 2))
    rr = np.array([(p - A[r] @ ut + 0.5) % 1 - 0.5 for r, p in enumerate(ph[:, 2])])
    for r in np.argsort(-np.abs(rr))[:12]: print('  outlier x %.5f resid %+.2f ampD %.0f ampS %.0f 2ulpS %.1e' % (ph[r, 0], rr[r], ph[r, 3], ph[r, 4], ph[r, 1]))

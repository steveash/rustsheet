"""D-chain phase solve with a small exact-CVP lattice (T-1941 W5).
Unknowns: integer ulp moves u3..u8 of D3..D8 (base: c5 with D3 -> ...b68, a multiple of 4 ulps).
Rows: tie-line phases of D.S3, D.S4, D.S5 in every region where S_j lands one binade above P_j.
A subsample of NROW rows builds the lattice (exact CVP); every row then checks the candidate.
usage: dsolve.py [control]   env: NROW (26), BOUND (1e6), TSIZE (control move size)"""
import numpy as np, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res, score
from metrics import XF, BF, XN, BN
from fpylll import IntegerMatrix, CVP, LLL
N0 = PK.N.copy(); D0 = PK.D.copy(); D0[3] = np.nextafter(D0[3], -np.inf)
IDX = [3, 4, 5, 6, 7, 8]
def withu(u):
    D = D0.copy()
    for k, i in enumerate(IDX): D[i] += u[k]*ulp(D0[i])
    return D
rows = []
for s in PK.segs:
    xm = PK.x[s][len(s)//2]; st, _, _ = stages(D0, xm)
    for j, P, dP, S, dS in st:
        if j in (3, 4, 5) and np.frexp(S)[1] == np.frexp(P)[1]+1: rows.append((s, xm, j, 2*ulp(S)))
def phases(B, D):
    out = []
    for s, xm, j, u2 in rows:
        xx = PK.x[s]; Sv = np.polyval(D0.astype(np.longdouble)[j:][::-1], xx.astype(np.longdouble))
        E = np.exp(-2j*np.pi*np.mod(Sv/np.longdouble(u2), 1.0).astype(float))*np.hanning(len(xx))
        d = PK.resid(xx, B[s]); m = PK.resid(xx, res(xx, N0, D))
        out.append(np.angle(np.dot(d-d.mean(), E)/np.dot(m-m.mean(), E))/2/np.pi)
    return np.array(out)
A = np.array([[ulp(D0[i])*xm**(i-j)/u2 if i >= j else 0.0 for i in IDX] for s, xm, j, u2 in rows])
control = len(sys.argv) > 1
allidx = np.concatenate(PK.segs); B = np.zeros_like(PK.x)
truth = None
if control:
    rng = np.random.default_rng(11); T = int(float(os.environ.get('TSIZE', 3e4)))
    truth = rng.integers(-T, T, len(IDX)); truth[0] = 4*(truth[0]//4)
    B[allidx] = res(PK.x[allidx], N0, withu(truth))
else: B[allidx] = PK.b[allidx]
ph = phases(B, D0)
print('rows', len(rows), 'by stage', {j: sum(1 for r in rows if r[2] == j) for j in (3, 4, 5)})
if truth is not None:
    pr = (ph - A @ truth + 0.5) % 1 - 0.5; print('truth phase rms %.3f' % pr.std())
NROW = int(os.environ.get('NROW', 26)); BOUND = float(os.environ.get('BOUND', 1e6))
sel = np.linspace(0, len(rows)-1, NROW).astype(int)
SC = 10**7; M = 30.0; K = len(IDX)
L = [[int(round(SC/BOUND)) if kk == k else 0 for kk in range(K)] + [int(round(SC*M*A[r, k])) for r in sel] for k in range(K)]
L += [[0]*K + [int(round(SC*M)) if rr == r else 0 for rr in sel] for r in sel]
Bm = IntegerMatrix.from_matrix(L); LLL.reduction(Bm)
for sgn in (1, -1):
    tgt = [0]*K + [int(round(SC*M*sgn*ph[r])) for r in sel]
    v = CVP.closest_vector(Bm, tgt, method='proved')
    u = np.array([round(v[k]*BOUND/SC) for k in range(K)])
    pr = (sgn*ph - A @ u + 0.5) % 1 - 0.5
    print('sign %+d: u = %s  phase rms all rows %.3f (lattice rows %.3f)' % (sgn, u.tolist(), pr.std(), pr[sel].std()), flush=True)
    if truth is not None: print('   truth  %s  recovered: %s' % (truth.tolist(), bool(np.all(u == truth))))
    else:
        D1 = withu(u); print('   exact: base far %.4f near %.4f -> far %.4f near %.4f' % (score(XF, BF, N0, D0), score(XN, BN, N0, D0), score(XF, BF, N0, D1), score(XN, BN, N0, D1)))

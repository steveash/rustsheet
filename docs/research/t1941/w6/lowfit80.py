# T-1941 W6: low block under the mixed model (upper Horner stages stored, lit; everything below a hand-off stage kept in
# 80-bit; R = st(N80/D80), N and D never stored). Given the stored hand-off values A(x) (N: S_JN, D: P_JD or S_JD) the
# model is linear in the low coefficients: N80 = A_N x^JN + sum_{j<JN} c_j x^j, likewise D. Rows R_data D80 - N80 = 0
# (relative), solved in mpmath at NPTS points, coefficients rounded to double; then exact R of the KEEP model.
# Usage: lowfit80.py RFILE CHAIN OUT [npts]   env HN (N hand-off, e.g. S4), HD (e.g. P2), FIX (low coefs held, e.g. D0)
import sys, os, numpy as np, mpmath as mp

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain, nrm, simR

mp.mp.dps = 40
rf, chain, out = sys.argv[1:4]
npts = int(sys.argv[4]) if len(sys.argv) > 4 else 4000
HN, HD = os.environ.get("HN", "S4"), os.environ.get("HD", "P2")
FIX = set(os.environ.get("FIX", "D0").split(",")) - {""}
N, D = read_chain(chain)
N = [v / nrm(D) for v in N]
D = [v / nrm(D) for v in D]
rows = [l.split() for l in open(rf)]
rng = np.random.default_rng(5)
sel = rng.choice(len(rows), npts, replace=False)
x = np.array([float.fromhex(rows[i][0]) for i in sel])
Rd = [(mp.mpf(float.fromhex(rows[i][2])) + mp.mpf(float.fromhex(rows[i][3]))) / 2 for i in sel]


def handoff(C, h):
    """stored value of hand-off site h (e.g. S4 / P2) per x, and the power of x it multiplies, and the free low js"""
    op, J = h[0], int(h[1:])
    s = L(C[-1]) * np.ones(len(x), dtype=L)
    XL = x.astype(L)
    for j in range(len(C) - 2, J - 1, -1):
        p = L(st(s * XL))
        if op == "P" and j == J:
            return p, J, list(range(J - 1, -1, -1)) + [J]  # P_J stored: S_J = P_J + c_J in 80-bit, c_J free too
        s = L(st(p + L(C[j])))
    return s, J, list(range(J - 1, -1, -1))


def keepset(ch, C, h):
    op, J = h[0], int(h[1:])
    k = ["%s.S%d" % (ch, J)] if op == "P" else []
    for j in range(J - 1, -1, -1):
        k += ["%s.P%d" % (ch, j), "%s.S%d" % (ch, j)]
    return k


AN, JN, fn = handoff(N, HN)
AD, JD, fd = handoff(D, HD)
fn = [j for j in fn if "N%d" % j not in FIX]
fd = [j for j in fd if "D%d" % j not in FIX]
cols, b = [], []
for i in range(npts):
    X = mp.mpf(x[i])
    n0 = mp.mpf(float(AN[i])) * X**JN + sum(mp.mpf(N[j]) * X**j for j in range(JN) if j not in fn)
    if HN[0] == "P" and JN not in fn:
        n0 += mp.mpf(N[JN]) * X**JN
    d0 = mp.mpf(float(AD[i])) * X**JD + sum(mp.mpf(D[j]) * X**j for j in range(JD) if j not in fd)
    if HD[0] == "P" and JD not in fd:
        d0 += mp.mpf(D[JD]) * X**JD
    sc = Rd[i] * d0
    # unknowns: N_j (j in fn) and D_j (j in fd): N80 - Rd D80 = 0
    cols.append([X**j / sc for j in fn] + [-Rd[i] * X**j / sc for j in fd])
    b.append(-(n0 - Rd[i] * d0) / sc)
A = mp.matrix(cols)
bb = mp.matrix(b)
# scaled normal equations via QR in mpmath
sol = mp.qr_solve(A, bb)[0]
for k, j in enumerate(fn):
    N[j] = float(sol[k])
for k, j in enumerate(fd):
    D[j] = float(sol[len(fn) + k])
res = [float((sum(A[i, k] * sol[k] for k in range(A.cols)) - bb[i])) for i in range(npts)]
print("free N%s D%s; residual rms %.2e" % (fn, fd, np.sqrt(np.mean(np.square(res)))))
open(out, "w").write("N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n")
keep = keepset("N", N, HN) + keepset("D", D, HD)
print("KEEP=" + ",".join(keep))

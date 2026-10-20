# T-1941 W6: exact-R scan of one coefficient along the function-weak direction. For each offset t (ulps) of the scanned
# coefficient, the coupled coefficients absorb its function change by least squares on the function rows (latsolve.func_rows, or
# Excel-R-anchored data_rows with DATAF), rounded to whole ulps, and the chain is scored by the exact-R rate. A
# coordinate climb cannot move along this direction (every single step breaks the function); this follows it.
# Usage: nullscan.py RFILE CHAIN OUT SCAN "COUPLED,..." TMIN TMAX STEPS [npts]   e.g. N9 "N5,N6,N7,N8,D5,D6,D7,D8,D9"
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L, horner
from synth import read_chain, nrm
from latsolve import func_rows, data_rows

rf, chain, out, scan, coupled = sys.argv[1:6]
tmin, tmax, steps = float(sys.argv[6]), float(sys.argv[7]), int(sys.argv[8])
npts = int(sys.argv[9]) if len(sys.argv) > 9 else 20000
N, D = read_chain(chain)
N = [v /nrm(D) for v in N]
D = [v /nrm(D) for v in D]
nN = len(N)
names = ["N%d" % k for k in range(nN)] + ["D%d" % k for k in range(1, len(D))]
cs = np.array(N + D[1:])
u = np.array([np.spacing(abs(v)) for v in cs])
fr = (
    data_rows(N, D, os.environ["DATAF"], int(os.environ.get("DATAN", "400")))
    if os.environ.get("DATAF")
    else func_rows(N, D)
)
A = np.array([[float(v) for v in g] for g, _ in fr])
b = -np.array([float(v) for _, v in fr])
si = names.index(scan)
ci = [names.index(c) for c in coupled.split(",")]
As, Ac = A[:, si], A[:, ci]
sc_ = np.max(np.abs(Ac), axis=0)
P = np.linalg.pinv(Ac / sc_)
rows = [l.split() for l in open(rf)]
idx = np.random.default_rng(0).choice(len(rows), min(npts, len(rows)), replace=False)
x = np.array([float.fromhex(rows[i][0]) for i in idx])
lo = np.array([float.fromhex(rows[i][2]) for i in idx])
hi = np.array([float.fromhex(rows[i][3]) for i in idx])


def chain_at(t):
    m = np.zeros(len(cs))
    m[si] = round(t)
    m[ci] = np.round((P @ (-As * m[si])) / sc_)  # follow the null direction from the current chain (its own residual kept)
    c = cs + m * u
    return list(c[:nN]), [1.0] + list(c[nN:]), m


def score(Nc, Dc):
    R = st(L(horner(Nc[::-1], x)) / L(horner(Dc[::-1], x)))
    return np.mean((R >= lo) & (R <= hi))


best = (-1, None)
for t in np.linspace(tmin, tmax, steps):
    Nc, Dc, m = chain_at(t)
    s = score(Nc, Dc)
    if s > best[0]:
        best = (s, t, Nc, Dc, m)
print(
    "%s scan [%g, %g] x %d, coupled %s: best t %+.0f exact %.4f"
    % (scan, tmin, tmax, steps, coupled, best[1], best[0])
)
print("  offsets:", " ".join("%s %+d" % (names[k], int(best[4][k])) for k in [si] + ci))
open(out, "w").write(
    "N "
    + " ".join(float(v).hex() for v in best[2])
    + "\nD "
    + " ".join(float(v).hex() for v in best[3])
    + "\n"
)

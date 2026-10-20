# T-1941 W6: final polish of a linear top stage (S8 = c8 + c9 x of a degree-9 chain) by exact-R rate on a 2-D grid of
# its change dS8(x) = (e0 + e1 (x - X0)) u_S8: c9 moves by e1 u_S8 / u9 ulps, c8 by (e0 - e1 X0) u_S8 / u8. Coarse
# grid, then refine around the best. Once the stages below are right, S8 enters R only through ~0.001-ulp roundings,
# so a whole-output objective is the right finisher here (the site objectives cannot see it).
# Usage: grid8.py RFILE CHAIN OUT CH E0MAX E1MAX [STEP0 STEP1] [npts]   env X0 (1.75)
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from synth import read_chain, simR, nrm

rf, chain, out, CH = sys.argv[1:5]
E0, E1 = float(sys.argv[5]), float(sys.argv[6])
s0 = float(sys.argv[7]) if len(sys.argv) > 7 else 1.0
s1 = float(sys.argv[8]) if len(sys.argv) > 8 else 2.0
npts = int(sys.argv[9]) if len(sys.argv) > 9 else 40000
X0 = float(os.environ.get("X0", 1.75))
rows = [l.split() for l in open(rf)]
idx = np.random.default_rng(1).choice(len(rows), min(npts, len(rows)), replace=False)
x = np.array([float.fromhex(rows[i][0]) for i in idx])
lo = np.array([float.fromhex(rows[i][2]) for i in idx])
hi = np.array([float.fromhex(rows[i][3]) for i in idx])
N, D = read_chain(chain)
N = list(np.array(N) /nrm(D))
D = list(np.array(D) /nrm(D))
C = N if CH == "N" else D
top = len(C) - 1
uS = np.spacing(abs(C[top - 1] + C[top] * X0))
u8, u9 = np.spacing(abs(C[top - 1])), np.spacing(abs(C[top]))
base8, base9 = C[top - 1], C[top]


def chain_at(e0, e1):
    k9 = round(e1 * uS / u9)
    k8 = round((e0 * uS - k9 * u9 * X0) / u8)
    C[top - 1], C[top] = base8 + k8 * u8, base9 + k9 * u9
    return k8, k9


def score():
    R = simR(N, D, x)
    return np.mean((R >= lo) & (R <= hi))


best = (score(), 0, 0, 0.0, 0.0)
print("start %.5f" % best[0], flush=True)
for e0 in np.arange(-E0, E0 + 1e-9, s0):
    for e1 in np.arange(-E1, E1 + 1e-9, s1):
        k8, k9 = chain_at(e0, e1)
        v = score()
        if v > best[0]:
            best = (v, k8, k9, e0, e1)
print(
    "coarse best %.5f at e0 %.1f e1 %.1f (%s%d %+d, %s%d %+d)"
    % (best[0], best[3], best[4], CH, top - 1, best[1], CH, top, best[2]),
    flush=True,
)
for e0 in np.arange(best[3] - s0, best[3] + s0 + 1e-9, s0 / 4):
    for e1 in np.arange(best[4] - s1, best[4] + s1 + 1e-9, s1 / 4):
        k8, k9 = chain_at(e0, e1)
        v = score()
        if v > best[0]:
            best = (v, k8, k9, e0, e1)
C[top - 1], C[top] = base8 + best[1] * u8, base9 + best[2] * u9
print("fine best %.5f (%s%d %+d, %s%d %+d)" % (best[0], CH, top - 1, best[1], CH, top, best[2]), flush=True)
open(out, "w").write(
    "N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n"
)

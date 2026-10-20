# T-1941 W6: chain-top polish with W5's exact objective (hypo.c via top6.py; J0 = top-2, withlast = 0, i.e. scored on
# the sites S_top-1, P_top-2, S_top-2 only, independent of c_J0's whole-u shifts): a coarse (c_top-1, c_top) grid, then
# a fine one around the best. Usage: topgrid6.py RFILE CHAIN OUT CH [TRUTH]
#   env R1/S1 (c_top-1 half-range/step, 300/2), R2/S2 (c_top, 8e4/128), MAXRUNS (500), HYPOLIB
import sys, os, time, numpy as np
sys.path.insert(0, sys.path[0])
from top6 import Top6
from synth import read_chain
rf, chain, out, CH = sys.argv[1:5]
t = Top6(rf, chain, maxruns=int(os.environ.get("MAXRUNS", 500)))
C = (t.N if CH == "N" else t.D).copy()
top = len(C) - 1
J0 = top - 2
u1, u2 = np.spacing(abs(C[top - 1])), np.spacing(abs(C[top]))
R1, S1 = int(float(os.environ.get("R1", 300))), int(float(os.environ.get("S1", 2)))
R2, S2 = int(float(os.environ.get("R2", 8e4))), int(float(os.environ.get("S2", 128)))
t0 = time.time()
k1 = np.arange(-R1, R1 + 1, S1)
k2 = np.arange(-R2, R2 + 1, S2)
H = np.array([[C[J0], C[top - 1] + a * u1, C[top] + b * u2] for a in k1 for b in k2])
v = t.obj(CH, J0, H, 0)
o = np.argsort(v)[::-1]
mu, sd = np.median(v), v.std()
print("coarse %d hyps (%d runs, %d pts) %.0fs; top: %s" % (len(H), t.nr, len(t.x), time.time() - t0, " | ".join("d%d %+d d%d %+d %.1f (Z %.1f)" % (top - 1, k1[i // len(k2)], top, k2[i % len(k2)], v[i], (v[i] - mu) / sd) for i in o[:4])), flush=True)
b1, b2 = k1[o[0] // len(k2)], k2[o[0] % len(k2)]
f1 = np.arange(b1 - S1, b1 + S1 + 1)
f2 = np.arange(b2 - S2, b2 + S2 + 1, max(1, S2 // 16))
Hf = np.array([[C[J0], C[top - 1] + a * u1, C[top] + b * u2] for a in f1 for b in f2])
vf = t.obj(CH, J0, Hf, 0)
i = int(np.argmax(vf))
d1, d2 = f1[i // len(f2)], f2[i % len(f2)]
print("fine: d%d %+d d%d %+d %.1f" % (top - 1, d1, top, d2, vf[i]), flush=True)
N, D = read_chain(chain)
N = list(np.array(N) / D[0])
D = list(np.array(D) / D[0])
arr = N if CH == "N" else D
arr[top - 1] += d1 * u1
arr[top] += d2 * u2
open(out, "w").write("N " + " ".join(float(v_).hex() for v_ in N) + "\nD " + " ".join(float(v_).hex() for v_ in D) + "\n")
if len(sys.argv) > 5:
    Nt, Dt = read_chain(sys.argv[5])
    Ct = (np.array(Nt) if CH == "N" else np.array(Dt)) / Dt[0]
    print("truth offsets: d%d %+d d%d %+d; truth score %.1f" % (top - 1, round((Ct[top - 1] - C[top - 1]) / u1), top, round((Ct[top] - C[top]) / u2), t.obj(CH, J0, np.array([[C[J0], Ct[top - 1], Ct[top]]]), 0)[0]))

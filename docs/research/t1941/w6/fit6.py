# T-1941 W6: turn a top-stage scan result into coefficients by least squares instead of literally (apply6.py). The
# scan pins the S_J rounding phase where the data is, but its polynomial coefficients are correlated (a grid step in d
# moves c_top by ~1e7 ulps), so: unknowns dc_J .. dc_top (ulps) and the whole-u integer m, rows
#   phase rows   dS_J(x_i)/u - poly(x_i - X0) - m = 0 at NPH points spread over the data's x range (sigma SPH, 0.05)
#   function rows (latsolve.func_rows; DATAF = Excel-R-anchored data_rows) for the full chain (sigma SF, 1e-16)
# m comes out continuous and is rounded, then dc is re-solved with m fixed. Usage:
#   fit6.py BASE OUT CH J a b [c [d]]   env X0 (1.75), XLO/XHI (1.5/2.0), NPH (200), SPH, SF, DATAF, DATAN,
#   FREE (e.g. D5,D6,D7,D8,D9: coefficients free in the function rows only)
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from synth import read_chain, nrm
from latsolve import func_rows, data_rows

base, out, CH, J = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4])
co = np.array([float(v) for v in sys.argv[5:]])
X0 = float(os.environ.get("X0", 1.75))
SPH, SF = float(os.environ.get("SPH", 0.05)), float(os.environ.get("SF", 1e-16))
N, D = read_chain(base)
N = list(np.array(N) /nrm(D))
D = list(np.array(D) /nrm(D))
C = N if CH == "N" else D
top = len(C) - 1
nN = len(N)
off = (
    0 if CH == "N" else nN - 1
)  # index of c_0 of this chain in the 19-vector (D0 is fixed, so D index j -> nN - 1 + j)
s = C[-1]
for j in range(top - 1, J - 1, -1):
    s = s * X0 + C[j]
u = float(np.spacing(abs(s)))
ks = list(range(J, top + 1))
uk = np.array([np.spacing(abs(C[k])) for k in ks])
xs = np.linspace(
    float(os.environ.get("XLO", 1.5)), float(os.environ.get("XHI", 2.0)), int(os.environ.get("NPH", 200))
)
# phase rows: sum_k dc_k u_k x^(k-J) / u - m = poly(x - X0)
Ap = np.array([[uk[i] * x ** (k - J) / u for i, k in enumerate(ks)] + [-1.0] for x in xs]) / SPH
bp = np.array([np.polyval(co[::-1], x - X0) for x in xs]) / SPH
fr = (
    data_rows(N, D, os.environ["DATAF"], int(os.environ.get("DATAN", "400")))
    if os.environ.get("DATAF")
    else func_rows(N, D)
)
cols = [(k if CH == "N" else off + k) for k in ks]
# extra coefficients free in the function rows only (e.g. the other chain's top, which compensates in the base chain)
FREE = [f for f in os.environ.get("FREE", "").split(",") if f]
fcols = [(int(f[1:]) if f[0] == "N" else nN - 1 + int(f[1:])) for f in FREE]
Af = np.array([[float(g[c]) for c in cols] + [0.0] + [float(g[c]) for c in fcols] for g, _ in fr]) / SF
Ap = np.hstack([Ap, np.zeros((len(Ap), len(fcols)))])
bf = -np.array([float(v) for _, v in fr]) / SF
A = np.vstack([Ap, Af])
b = np.concatenate([bp, bf])
sc = np.max(np.abs(A), axis=0)
z, *_ = np.linalg.lstsq(A / sc, b, rcond=None)
z /= sc
im = len(ks)
m = round(z[im])
# re-solve with m fixed
A2 = np.delete(A, im, axis=1)
b2 = b - A[:, im] * m
sc2 = np.max(np.abs(A2), axis=0)
z2, *_ = np.linalg.lstsq(A2 / sc2, b2, rcond=None)
z2 /= sc2
res = A2 @ z2 - b2
print(
    "m %+d (continuous %.2f); phase rms %.3f u, function rms %.1e"
    % (m, z[-1], np.sqrt(np.mean((res[: len(xs)] * SPH) ** 2)), np.sqrt(np.mean((res[len(xs) :] * SF) ** 2)))
)
for i, k in enumerate(ks):
    C[k] = C[k] + round(z2[i]) * uk[i]
    print("  %s%d %+d ulps" % (CH, k, round(z2[i])))
for i, f in enumerate(FREE):
    arr = N if f[0] == "N" else D
    j = int(f[1:])
    v = round(z2[len(ks) + i])
    arr[j] = arr[j] + v * np.spacing(abs(arr[j]))
    print("  %s %+d ulps (function only)" % (f, v))
open(out, "w").write(
    "N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n"
)

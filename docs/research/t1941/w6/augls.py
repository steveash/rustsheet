# T-1941 W6: augmented least squares against Excel's own R: keep a chain's phase-pinned coefficients, re-solve the free
# low ones, and add ONE extra parameter for a candidate outer form; the right form drops the residual to the data
# noise (~2e-16). Rows at NPTS points: N(x) - Rd(x) D(x) - p * T(x) = 0, relative to Rd D, with T per form:
#   none; k (R = k N/D): T = Rd D; c (R = N/D + c): T = D; cx2 / cx1 (R = N/D + c(x-2) / c(x-1)): T = (x-2) D / (x-1) D;
#   e (B = (x-2) N/D (x-1+e)): T = -N/(x-1); xN10 / xN11 / xD10: an extra leading coefficient x^10 / x^11 (N) or x^10 (D)
# Usage: augls.py RFILE CHAIN "N0,N1,N2,N3,N4,D1,D2" [npts]   (prints the residual for every form)
import sys, numpy as np, mpmath as mp

sys.path.insert(0, sys.path[0])
from synth import read_chain, nrm

mp.mp.dps = 40
rf, chain, free = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
npts = int(sys.argv[4]) if len(sys.argv) > 4 else 2000
N, D = read_chain(chain)
N = [v /nrm(D) for v in N]
D = [v /nrm(D) for v in D]
rows = [l.split() for l in open(rf)]
idx = np.random.default_rng(7).choice(len(rows), npts, replace=False)
X = [mp.mpf(float.fromhex(rows[i][0])) for i in idx]
Rd = [(mp.mpf(float.fromhex(rows[i][2])) + mp.mpf(float.fromhex(rows[i][3]))) / 2 for i in idx]
Nv = [mp.polyval([mp.mpf(v) for v in N[::-1]], x) for x in X]
Dv = [mp.polyval([mp.mpf(v) for v in D[::-1]], x) for x in X]
sc = [r * d for r, d in zip(Rd, Dv)]
# base residual and the free columns (per ulp of each coefficient)
b = np.array([float(-(n - r * d) / s) for n, r, d, s in zip(Nv, Rd, Dv, sc)])
cols = []
for f in free:
    j = int(f[1:])
    u = mp.mpf(np.spacing(abs(N[j] if f[0] == "N" else D[j])))
    cols.append([float((x**j * u if f[0] == "N" else -r * x**j * u) / s) for x, r, s in zip(X, Rd, sc)])
forms = {
    "none": None,
    "k": [float(r * d / s) for r, d, s in zip(Rd, Dv, sc)],
    "c": [float(d / s) for d, s in zip(Dv, sc)],
    "cx2": [float((x - 2) * d / s) for x, d, s in zip(X, Dv, sc)],
    "cx1": [float((x - 1) * d / s) for x, d, s in zip(X, Dv, sc)],
    "e": [float(-n / (x - 1) / s) for x, n, s in zip(X, Nv, sc)],
    "xN10": [float(x**10 / s) for x, s in zip(X, sc)],
    "xN11": [float(x**11 / s) for x, s in zip(X, sc)],
    "xD10": [float(-r * x**10 / s) for x, r, s in zip(X, Rd, sc)],
}
for name, extra in forms.items():
    A = np.array(cols + ([extra] if extra else [])).T
    s_ = np.max(np.abs(A), axis=0)
    z, *_ = np.linalg.lstsq(A / s_, b, rcond=None)
    r = A @ (z / s_) - b
    print(
        "  %-5s residual rms %.2e  max %.2e%s"
        % (
            name,
            np.sqrt(np.mean(r**2)),
            np.abs(r).max(),
            ("   extra param %.3e" % (z[-1] / s_[-1])) if extra else "",
        )
    )

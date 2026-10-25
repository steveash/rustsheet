"""Line spectrum of Excel's density error along a consecutive-double run (FFT of err - mean), top peaks, and the
sawtooth frequencies predicted for candidate stored intermediates Q (f = frac(Q'(x) ulp(x) / ulp(Q)), folded)."""
import math
import sys
import numpy as np
import mpmath
import dlib as L

rows = L.load()
blk = sys.argv[1]
idx = sorted(L.blocks(rows)[blk], key=lambda i: rows[i]["x"])
e = np.array([L.err_units(rows[i]["excel"], rows[i]["exact"]) for i in idx])
e = e - e.mean()
P = np.abs(np.fft.rfft(e)) ** 2
fr = np.fft.rfftfreq(len(e))
top = np.argsort(P)[::-1][:12]
print(blk, "top lines (freq cycles/step, power share):")
tot = P[1:].sum()
for j in top:
    print(f"   f={fr[j]:.4f}  share {P[j] / tot:.3f}")
r0 = rows[idx[len(idx) // 2]]
x, a, b = r0["x"], r0["a"], r0["b"]
ux = math.ulp(x)
k, n = a - 1, a + b - 2
nk = n - k
y = 1 - x
npx, nqx = n * x, n * y
cands = {
    "E=ln pdf": (math.log(r0["exact"]), (a - 1) / x - (b - 1) / y),
    "(a-1)lnx": ((a - 1) * math.log(x), (a - 1) / x),
    "(b-1)lny": ((b - 1) * math.log(y), -(b - 1) / y),
    "lnx": (math.log(x), 1 / x),
    "lny": (math.log(y), -1 / y),
    "y": (y, -1.0),
    "np": (npx, n),
    "nq": (nqx, -n),
    "k/np": (k / npx, -k / (npx * x)),
    "nk/nq": (nk / nqx, nk / (nqx * y)),
    "tp=k ln(k/np)": (k * math.log(k / npx), -k / x),
    "tq": (nk * math.log(nk / nqx), nk / y),
    "k+np": (k + npx, n),
    "nk+nq": (nk + nqx, -n),
    "tp+np": (k * math.log(k / npx) + npx, -k / x + n),
    "tq+nq": (nk * math.log(nk / nqx) + nqx, nk / y - n),
    "x*y": (x * y, 1 - 2 * x),
    "a lnx": (a * math.log(x), a / x),
    "b lny": (b * math.log(y), -b / y),
    "a lnx + b lny": (a * math.log(x) + b * math.log(y), a / x - b / y),
    "lambda=a-(a+b)x": (a - (a + b) * x, -(a + b)),
    "(a+b)x": ((a + b) * x, a + b),
}
print("candidate sawtooth frequencies:")
for name, (v, dv) in cands.items():
    f = abs(dv * ux / math.ulp(v)) % 1.0
    f = min(f, 1 - f)
    print(f"   {name:18s} |Q|={abs(v):.4g} f={f:.4f}")

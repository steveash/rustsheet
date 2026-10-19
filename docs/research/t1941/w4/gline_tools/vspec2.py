# 2-D (v,w) incoherent spectrum via zero-padded FFT lookups. local freq = f + 2 v eps n + 3 w eps^2 n^2 (n = run centre)
# usage: vspec2.py f [e3res.txt] [NMIN]   (e3res: y_hex excel_hex residual_in_2^-53)
import numpy as np, sys
from fractions import Fraction as Fr

f = float(sys.argv[1])
E3 = sys.argv[2] if len(sys.argv) > 2 else "../e3res.txt"
NMIN = float(sys.argv[3]) if len(sys.argv) > 3 else 3e11
L = [l.split() for l in open(E3)]
y = np.array([float.fromhex(a) for a, b, c in L])
r = np.array([float(c) for a, b, c in L])
brk = [0] + [i for i in range(1, len(y)) if not (0 < y[i] - y[i - 1] < 1e-12)] + [len(y)]
NF = 1 << 19
eps = 2.0**-52
specs = []
for a, b in zip(brk[:-1], brk[1:]):
    if y[a] > 1.5:
        continue
    n0 = int((Fr(y[a]) - 1) / Fr(2) ** -52)
    if n0 <= NMIN:
        continue
    rr = r[a:b] - r[a:b].mean()
    rr = rr * np.hanning(len(rr))
    P = np.abs(np.fft.fft(rr, NF)) ** 2
    P /= np.median(P)
    specs.append((n0 + len(rr) / 2, P))
vs = np.arange(-14, 14, 0.004)
ws = np.arange(
    -40, 40.01, 0.5
)  # vspec3.py: same, with a fine grid vs=v0±0.06 step 2e-4, ws=w0±4 step 0.05 (argv 4,5)
V, Wg = np.meshgrid(vs, ws, indexing="ij")
tot = np.zeros(V.shape)
for n, P in specs:
    for h in (1, 2):
        fl = (h * (f + 2 * V * eps * n + 3 * Wg * eps**2 * n * n)) % 1
        tot += P[(np.round(fl * NF).astype(int)) % NF] / h
tot /= len(specs)
flat = np.argsort(-tot, axis=None)
pk = []
for i in flat:
    a, b = np.unravel_index(i, tot.shape)
    if all(abs(vs[a] - p[0]) > 0.08 for p in pk):
        pk.append((vs[a], ws[b], tot[a, b]))
    if len(pk) == 6:
        break
print(
    "f=%s runs=%d " % (sys.argv[1], len(specs)) + "  ".join("v=%+.3f w=%+.1f (%.1f)" % p for p in pk),
    flush=True,
)

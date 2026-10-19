# per-stage data-vs-sim phase on the e3 ladder; prints (a,b,c) deltas of the stage functional
# usage: stagephase4.py fit.txt CHAIN j saw|tri simres.txt [KMAX] [e3res.txt]
import numpy as np, sys, math
from fractions import Fraction as Fr

coef = [Fr(float.fromhex(h)) for h in open(sys.argv[1]).read().split()]
N = coef[:9]
D = coef[9:]
chain, j, typ = (
    sys.argv[2],
    int(sys.argv[3]),
    sys.argv[4],
)  # saw: multiplicand M_j (=S_{j+1}) * y ; tri: sum S_j
C = N if chain == "N" else D


def Sder(jj, k):
    return sum(C[i] * math.comb(i - jj, k) for i in range(jj, len(C)))


if typ == "saw":
    jj = j + 1
    e = math.frexp(float(Sder(jj, 0)))[1] - 1
    lawc = [Sder(jj, 0) / 2**e, Sder(jj, 1) / 2**e, Sder(jj, 2) / 2**e]
else:
    jj = j
    e = math.frexp(float(Sder(jj, 0)))[1] - 1
    lawc = [Sder(jj, 1) / 2 ** (e + 1), Sder(jj, 2) / 2 ** (e + 1), Sder(jj, 3) / 2 ** (e + 1)]
lawc = [float(x) for x in lawc]
D3 = [l.split() for l in open(sys.argv[7] if len(sys.argv) > 7 else "../e3res.txt")]
S3 = [l.split() for l in open(sys.argv[5] if len(sys.argv) > 5 else "e3sim.txt")]
y = np.array([float.fromhex(a) for a, b, c in D3])
rd = np.array([float(c) for a, b, c in D3])
rs = np.array([float(b) for a, b in S3])
brk = [0] + [i for i in range(1, len(y)) if not (0 < y[i] - y[i - 1] < 1e-12)] + [len(y)]
eps = 2.0**-52
out = []
for a, b in zip(brk[:-1], brk[1:]):
    if y[a] > 1.5:
        continue
    n0 = int((Fr(y[a]) - 1) / Fr(2) ** -52)
    K = (n0 + np.arange(b - a)).astype(np.longdouble)
    ph = np.mod(
        sum(np.longdouble(lawc[k]) * np.longdouble(eps) ** k * K ** (k + 1) for k in range(3)), 1.0
    ).astype(float)
    Ez = np.exp(-2j * np.pi * ph) * np.hanning(b - a)
    d = rd[a:b] - rd[a:b].mean()
    s = rs[a:b] - rs[a:b].mean()
    Ad = np.dot(d, Ez)
    As = np.dot(s, Ez)
    out.append((float(n0), abs(Ad), abs(As), np.angle(Ad / As) / 2 / np.pi))
Kv = np.array([o[0] for o in out])
ph = np.array([o[3] for o in out])


def basis(k, o):
    cols = [k * (eps * k) ** jj for jj in range(o)]
    if typ == "tri":
        cols = [np.ones_like(k)] + cols
    return np.array(cols).T


unw = ph.copy()
used = Kv < 1e6
KM = float(sys.argv[6]) if len(sys.argv) > 6 else 3.2e12
for kmax in [1e6 * 2**i for i in range(0, 29)]:
    if kmax > KM:
        break
    o = 1 if kmax < 1e10 else (2 if kmax < 1e11 else 3)
    B = basis(Kv[used], o)
    c, *_ = np.linalg.lstsq(B, unw[used], rcond=None)
    new = (Kv <= kmax) & ~used
    if new.any():
        unw[new] = ph[new] + np.round(basis(Kv[new], o) @ c - ph[new])
    used = Kv <= kmax
    B = basis(Kv[used], o)
    c, *_ = np.linalg.lstsq(B, unw[used], rcond=None)
    r = unw[used] - B @ c
cc = list(c[1:]) if typ == "tri" else list(c)
print(
    "rms %.4f deltas a=%+.5e b=%+.5e c=%+.5e (x2^%d)"
    % (r.std(), cc[0], cc[1], cc[2] if len(cc) > 2 else 0, e if typ == "saw" else e + 1)
)

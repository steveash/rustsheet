"""Pinned refit for degree reduction (T-1941 W5 part 3, W4's method).
Start from c5_f99t (with Excel's tie-line bits: D3 -> multiple of 4 ulps, N2 even), drop the requested top
coefficients, and solve a weighted least-squares correction of the remaining ones that keeps
  (a) the function N/D on [0.7, 1.5]                      (tolerance FTOL relative, default 1e-17)
  (b) every Horner stage's Taylor values at x = 1, orders 0, 1, 2, as in c5 (near-1 pins)
      (tolerances relative to the stage: 2e-16, 1e-13, 1e-10)
D0 stays 2^-13. Writes the set to p3/pin_<tag>.txt and prints exact far/near scores.
usage: pinfit.py TAG NDEG DDEG"""

import mpmath as mp, numpy as np, sys, os
from math import comb

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import candidate, score
from metrics import XF, BF, XN, BN

mp.mp.dps = 50
tag, nd, dd = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
FTOL = mp.mpf(os.environ.get("FTOL", "1e-17"))
TOL = [mp.mpf(v) for v in os.environ.get("STOL", "2e-16,1e-13,1e-10").split(",")]
N = candidate("N3").copy()
D = candidate("D3").copy()
D[3] = np.nextafter(D[3], -np.inf)
N[2] = np.nextafter(N[2], np.inf)
Nm = [mp.mpf(v) for v in N]
Dm = [mp.mpf(v) for v in D]


def stage_taylor(c, j, k):  # k-th Taylor coefficient at x=1 of S_j(x) = sum_{i>=j} c_i x^(i-j)
    return mp.fsum(c[i] * comb(i - j, k) for i in range(j, len(c)) if i - j >= k)


fN = list(range(nd + 1))
fD = list(range(1, dd + 1))  # free coefficients after the drop
nu = len(fN) + len(fD)


def newpoly(c, deg, delta):
    return [c[i] + delta[i] if i <= deg else mp.mpf(0) for i in range(len(c))][: deg + 1]


rows = []
rhs = []
xs = [mp.mpf("0.7") + mp.mpf("0.8") * k / 79 for k in range(80)]
for x in xs:
    Nv = mp.polyval(Nm[::-1], x)
    Dv = mp.polyval(Dm[::-1], x)
    A = Nv / Dv
    # dropped tops change N and D by -c_i x^i; corrections dN, dD must give (N+dN')/(D+dD') = A
    dropN = -mp.fsum(Nm[i] * x**i for i in range(nd + 1, len(Nm)))
    dropD = -mp.fsum(Dm[i] * x**i for i in range(dd + 1, len(Dm)))
    r = [x**i for i in fN] + [-A * x**i for i in fD]
    w = 1 / (FTOL * abs(Nv))
    rows.append([v * w for v in r])
    rhs.append(-(dropN - A * dropD) * w)
VIS = float(os.environ.get("VIS", 3e-18)); TT = mp.mpf(os.environ.get("TT", "1e-3"))
npin = 0
labels = ["func"] * len(rows)
for c, deg, free, off in ((Nm, nd, fN, 0), (Dm, dd, fD, len(fN))):
    full = abs(mp.fsum(c))
    for j in range(0, deg + 1):
        u = mp.mpf(np.spacing(abs(float(stage_taylor(c, j, 0)))))
        if u / full < VIS: continue          # this stage's rounding is invisible in the output
        for k in range(3):
            if j + k > deg: continue
            ref = stage_taylor(c, j, k); scale = 2 * u / TT**k * (float(os.environ.get("K0", 1)) if k == 0 else 1)
            r = [mp.mpf(0)] * nu
            for idx, i in enumerate(free):
                if i >= j and i - j >= k: r[off + idx] = comb(i - j, k) / scale
            new_without = mp.fsum(c[i] * comb(i - j, k) for i in range(j, deg + 1) if i - j >= k)
            rows.append(r); rhs.append((ref - new_without) / scale); npin += 1; labels.append("%s.S%d o%d" % ("N" if off == 0 else "D", j, k))
print("pin rows", npin)
Am = mp.matrix(rows)
bm = mp.matrix(rhs)
sol = mp.qr_solve(Am, bm)[0]
dN = {i: sol[k] for k, i in enumerate(fN)}
dD = {i: sol[len(fN) + k] for k, i in enumerate(fD)}
N2 = np.array([float(Nm[i] + dN[i]) for i in fN])
D2 = np.array([float(Dm[0])] + [float(Dm[i] + dD[i]) for i in fD])
res_ = Am * sol - bm
if os.environ.get('SHOW'):
    top = sorted(range(len(res_)), key=lambda i: -abs(res_[i]))[:12]
    print('   worst rows:', ', '.join('%s %.0f' % (labels[i], float(res_[i])) for i in top))
print(
    "%s (%d,%d): weighted resid rms %.2e  max %.2e"
    % (tag, nd, dd, float(mp.sqrt(mp.fsum(v**2 for v in res_) / len(res_))), float(max(abs(v) for v in res_)))
)
print(
    "   moves (ulps):",
    " ".join("N%d:%.0f" % (i, float(dN[i]) / np.spacing(abs(N[i]))) for i in fN),
    "|",
    " ".join("D%d:%.0f" % (i, float(dD[i]) / np.spacing(abs(D[i]))) for i in fD),
)
print("   exact far %.4f near %.4f" % (score(XF, BF, N2, D2), score(XN, BN, N2, D2)))
open(os.path.join(HERE, "pin_%s.txt" % tag), "w").write(
    " ".join(v.hex() for v in N2) + "\n" + " ".join(v.hex() for v in D2) + "\n"
)

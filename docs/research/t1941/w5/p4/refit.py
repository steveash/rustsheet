"""Step (4) of W-B (T-1941 W5 part 4): refit the function-visible directions with the plateau bit readings fixed.
Base: c5_f99t + bit moves (N0, N1, D1, D2, D3 read values; N2 +1 tie). Free: all other coefficients up to the
requested degree (top coefficients dropped for reduced degrees). Weighted least squares on
  (a) function rows: N/D equal to TARGET on 80 points of [0.7,1.5], tolerance FTOL relative;
      TARGET = c5's function (default) or c5's function times (1 + measured smooth bias) (env CORR=1);
  (b) near-1 stage pins: Taylor coefficients at x = 1 of every visible stage, orders 0-2, from c5,
      tolerance K0*2 ulp(S_j) for order 0 and 2 ulp(S_j)/TT^k for order k.
Rounds to doubles, then reports exact far/near and the staircase chi^2.
usage: refit.py TAG NDEG DDEG"""

import mpmath as mp, numpy as np, sys, os
from math import comb

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
tag, nd, dd = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
sys.argv = sys.argv[:1] + ["none"]
exec(open(os.path.join(HERE, "plateau.py")).read().split("if __name__")[0])
from sc import score
from metrics import XF, BF, XN, BN

mp.mp.dps = 50
MOVES = {("N", 1): -21, ("D", 3): -1, ("D", 2): 6, ("D", 1): 310, ("N", 0): 440, ("N", 2): 1}
FIXED = {("N", 0), ("N", 1), ("N", 2), ("D", 1), ("D", 2), ("D", 3)}
Nc = candidate("N3")
Dc = candidate("D3")
N0 = Nc.copy()
D0 = Dc.copy()
for (ch, k), j in MOVES.items():
    C = N0 if ch == "N" else D0
    C[k] += j * np.spacing(abs(C[k]))
Ncm = [mp.mpf(v) for v in Nc]
Dcm = [mp.mpf(v) for v in Dc]
Nb = [mp.mpf(v) for v in N0]
Db = [mp.mpf(v) for v in D0]
FTOL = mp.mpf(os.environ.get("FTOL", "1e-17"))
K0 = mp.mpf(os.environ.get("K0", "1000"))
TT = mp.mpf(os.environ.get("TT", "1e-6"))
free = [("N", i) for i in range(nd + 1) if ("N", i) not in FIXED] + [
    ("D", i) for i in range(1, dd + 1) if ("D", i) not in FIXED
]


def st_t(c, j, k):
    return mp.fsum(c[i] * comb(i - j, k) for i in range(j, len(c)) if i - j >= k)


rows = []
rhs = []
xs = [mp.mpf("0.7") + mp.mpf("0.8") * k / 79 for k in range(80)]
corr = None
if os.environ.get("CORR"):
    tmu, tse = runmean(b)
    mu, se = runmean(res(x, Nc, Dc))
    w_ = 1 / np.sqrt(tse**2 + se**2)
    cf, *_ = np.linalg.lstsq(V * w_[:, None], (tmu - mu) * w_, rcond=None)
    corr = lambda xx: float(np.polyval(cf[::-1] if False else cf, 0))  # placeholder, replaced below

    def corr(xx):
        u = (float(xx) - 1.1) / 0.4
        return sum(cf[i] * u ** (PDEG - i) for i in range(PDEG + 1)) * 2.0**-52


for xv in xs:
    tgt = mp.polyval(Ncm[::-1], xv) / mp.polyval(Dcm[::-1], xv)
    if corr:
        tgt *= 1 + mp.mpf(corr(xv))
    Nv = mp.polyval(Nb[::-1], xv)
    Dv = mp.polyval(Db[::-1], xv)
    # truncated base (drop tops above the degree)
    Nv_t = mp.fsum(Nb[i] * xv**i for i in range(nd + 1))
    Dv_t = mp.fsum(Db[i] * xv**i for i in range(dd + 1))
    r = [(xv**i if ch == "N" else -tgt * xv**i) for ch, i in free]
    w = 1 / (FTOL * abs(Nv))
    rows.append([v * w for v in r])
    rhs.append(-(Nv_t - tgt * Dv_t) * w)
for ch, cref, cb, deg in (("N", Ncm, Nb, nd), ("D", Dcm, Db, dd)):
    full = abs(mp.fsum(cref))
    for j in range(deg + 1):
        u = mp.mpf(np.spacing(abs(float(st_t(cref, j, 0)))))
        if u / full < 3e-18:
            continue
        for k in range(3):
            if j + k > deg:
                continue
            scale = 2 * u / TT**k * (K0 if k == 0 else 1)
            r = [mp.mpf(0)] * len(free)
            for n, (c2, i) in enumerate(free):
                if c2 == ch and i >= j and i - j >= k:
                    r[n] = comb(i - j, k) / scale
            have = mp.fsum(cb[i] * comb(i - j, k) for i in range(j, deg + 1) if i - j >= k)
            rows.append(r)
            rhs.append((st_t(cref, j, k) - have) / scale)
sol = mp.qr_solve(mp.matrix(rows), mp.matrix(rhs))[0]
N1 = np.array([float(Nb[i]) for i in range(nd + 1)])
D1 = np.array([float(Db[i]) for i in range(dd + 1)])
for (ch, i), v in zip(free, sol):
    if ch == "N":
        N1[i] = float(Nb[i] + v)
    else:
        D1[i] = float(Db[i] + v)
resid = mp.matrix(rows) * sol - mp.matrix(rhs)
tmu, tse = runmean(b)
print(
    "%s (%d,%d)%s: weighted resid rms %.2f max %.1f | exact far %.4f near %.4f | staircase chi2 %.0f"
    % (
        tag,
        nd,
        dd,
        " CORR" if corr else "",
        float(mp.sqrt(mp.fsum(v**2 for v in resid) / len(resid))),
        float(max(abs(v) for v in resid)),
        score(XF, BF, N1, D1),
        score(XN, BN, N1, D1),
        chi2(tmu, tse, N1, D1)[0],
    ),
    flush=True,
)
open(os.path.join(HERE, "refit_%s.txt" % tag), "w").write(
    " ".join(v.hex() for v in N1) + "\n" + " ".join(v.hex() for v in D1) + "\n"
)

"""Function correction to Excel's staircase-corrected curve (W-B step 4, T-1941 W5 part 4).
For a set S: fit the smooth polynomial p(x) (relative ulps) to per-run means of (Excel - sim_S) together with the
staircase (plateau.py chi^2 machinery); the target function is f_S(x)*(1 + p(x)*2^-52). Move only the unread
coefficients (default N3..N9, D4..D9) by least squares, with S's own near-1 stage Taylor pins (loose order 0),
round, and report exact scores and the staircase chi^2.
usage: fcorrect.py IN OUT"""

import mpmath as mp, numpy as np, sys, os
from math import comb

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
fin, fout = sys.argv[1], sys.argv[2]
sys.argv = sys.argv[:1] + ["none"]
exec(open(os.path.join(HERE, "plateau.py")).read().split("if __name__")[0])
from sc import score
from metrics import XF, BF, XN, BN

mp.mp.dps = 50
l = open(fin).read().split("\n")
N = np.array([float.fromhex(v) for v in l[0].split()])
D = np.array([float.fromhex(v) for v in l[1].split()])
tmu, tse = runmean(b)


def smooth_of(N, D):
    mu, se = runmean(res(x, N, D))
    d = tmu - mu
    w = 1 / np.sqrt(tse**2 + se**2)
    cf, *_ = np.linalg.lstsq(V * w[:, None], d * w, rcond=None)
    return cf


FREE = [("N", i) for i in range(3, len(N))] + [("D", i) for i in range(4, len(D))]
FTOL = mp.mpf(os.environ.get("FTOL", "1e-17"))
K0 = mp.mpf(os.environ.get("K0", "1000"))
TT = mp.mpf(os.environ.get("TT", "1e-6"))


def st_t(c, j, k):
    return mp.fsum(c[i] * comb(i - j, k) for i in range(j, len(c)) if i - j >= k)


for it in range(int(os.environ.get("ITERS", 2))):
    cf = smooth_of(N, D)
    p = lambda xv: sum(cf[i] * ((float(xv) - 1.1) / 0.4) ** (PDEG - i) for i in range(PDEG + 1)) * 2.0**-52
    Nm = [mp.mpf(v) for v in N]
    Dm = [mp.mpf(v) for v in D]
    rows = []
    rhs = []
    for k in range(80):
        xv = mp.mpf("0.7") + mp.mpf("0.8") * k / 79
        Nv = mp.polyval(Nm[::-1], xv)
        Dv = mp.polyval(Dm[::-1], xv)
        tgt = Nv / Dv * (1 + mp.sign((xv - 1) * Nv / Dv) * mp.mpf(p(xv)))  # bias is relative to res = t*A
        r = [(xv**i if ch == "N" else -tgt * xv**i) for ch, i in FREE]
        w = 1 / (FTOL * abs(Nv))
        rows.append([v * w for v in r])
        rhs.append(-(Nv - tgt * Dv) * w)
    for ch, cm in (("N", Nm), ("D", Dm)):
        full = abs(mp.fsum(cm))
        for j in range(len(cm)):
            u = mp.mpf(np.spacing(abs(float(st_t(cm, j, 0)))))
            if u / full < 3e-18:
                continue
            for kk in range(3):
                if j + kk >= len(cm):
                    continue
                scale = 2 * u / TT**kk * (K0 if kk == 0 else 1)
                r = [mp.mpf(0)] * len(FREE)
                for n, (c2, i) in enumerate(FREE):
                    if c2 == ch and i >= j and i - j >= kk:
                        r[n] = comb(i - j, kk) / scale
                rows.append(r)
                rhs.append(mp.mpf(0))
    sol = mp.qr_solve(mp.matrix(rows), mp.matrix(rhs))[0]
    for (ch, i), v in zip(FREE, sol):
        if ch == "N":
            N[i] = float(Nm[i] + v)
        else:
            D[i] = float(Dm[i] + v)
    print(
        "it %d: smooth level before (rel ulps) at 0.75/1.0/1.25/1.45: %s | exact far %.4f near %.4f | staircase chi2 %.0f"
        % (
            it,
            " ".join("%+.2f" % (p(v) / 2.0**-52) for v in (0.75, 1.0, 1.25, 1.45)),
            score(XF, BF, N, D),
            score(XN, BN, N, D),
            chi2(tmu, tse, N, D)[0],
        ),
        flush=True,
    )
open(os.path.join(HERE, fout), "w").write(
    " ".join(v.hex() for v in N) + "\n" + " ".join(v.hex() for v in D) + "\n"
)

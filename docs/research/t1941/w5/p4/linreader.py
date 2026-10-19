"""Linear plateau reader (T-1941 W5 part 4).
Per-run Delta_r = mean relative bias (target - candidate sim), relative ulps. Model:
  Delta(x) = smooth poly (PDEG) + sum_k w_k(x) * beta_k(seg_k(x)),
  w_k = x^j / N(x) for N stage j, -x^j / D(x) for D stage j, in relative ulps per unit of S (/2^-52),
  seg_k = segment between consecutive binade crossings of S_k or P_k (stage k's own configuration).
The first segment of each stage is the reference (a common shift is a function change, absorbed by the poly).
Prints beta_k(seg) - beta_k(seg0) in ulps of S_k on that segment, with SE.
usage: linreader.py control|real"""

import numpy as np, sys, os

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
sys.argv = sys.argv[:2] + ["0"]
src = open(os.path.join(HERE, "plateau.py")).read().split("if __name__")[0]
exec(src)
from events import events

mode = sys.argv[1]
if mode == "control":
    Nt = N5.copy()
    Dt = D5.copy()
    for (ch, k), j in {("N", 0): 37, ("D", 1): -11, ("D", 2): 3}.items():
        C = Nt if ch == "N" else Dt
        C[k] += j * np.spacing(abs(C[k]))
    tmu, tse = runmean(res(x, Nt, Dt))
else:
    tmu, tse = runmean(b)
mu, se = runmean(res(x, N5, D5))
d = tmu - mu
w = 1 / np.sqrt(tse**2 + se**2)
ev = events(N5, D5, LO, HI)
Nv = np.polyval(N5[::-1], xr)
Dv = np.polyval(D5[::-1], xr)
cols = [V]
labels = []
for ch, C, F in (("N", N5, 1 / Nv), ("D", D5, -1 / Dv)):
    for j in range(0 if ch == "N" else 1, len(C) - 1):
        cuts = sorted(xe for xe, nm, e0, e1 in ev if nm in ("%s.S%d" % (ch, j), "%s.P%d" % (ch, j)))
        if not cuts:
            continue
        seg = np.searchsorted(cuts, xr)
        for sgi in range(1, len(cuts) + 1):
            ind = seg == sgi
            if ind.sum() < 3:
                continue
            # scale: one ulp of S_j on this segment, so beta is read in ulps of S_j
            s = np.full_like(xr, C[-1])
            for i in range(len(C) - 2, j - 1, -1):
                s = s * xr + C[i]
            uS = np.spacing(np.abs(s))
            cols.append((F * xr**j * uS / 2**-52 * ind)[:, None])
            labels.append((ch, j, sgi, cuts[sgi - 1], float(np.median(uS[ind]))))
M = np.hstack(cols)
coef, *_ = np.linalg.lstsq(M * w[:, None], d * w, rcond=None)
r = (d - M @ coef) * w
cov = np.linalg.pinv((M * w[:, None]).T @ (M * w[:, None]))
print(
    "mode",
    mode,
    "runs",
    R,
    "chi2 before %.1f after %.1f, params %d"
    % (
        float(
            ((d - V @ np.linalg.lstsq(V * w[:, None], d * w, rcond=None)[0]) * w)
            @ ((d - V @ np.linalg.lstsq(V * w[:, None], d * w, rcond=None)[0]) * w)
        ),
        float(r @ r),
        M.shape[1],
    ),
)
for i, (ch, j, sgi, xc, uS) in enumerate(labels):
    k = V.shape[1] + i
    z = coef[k] / np.sqrt(cov[k, k])
    if abs(z) > float(os.environ.get("ZMIN", 4)):
        print(
            "  %s.S%d seg %d (from x=%.4f, ulp S %.1e): beta-beta0 = %+.3f +- %.3f ulps(S)  z %+.1f"
            % (ch, j, sgi, xc, uS, coef[k], np.sqrt(cov[k, k]), z)
        )

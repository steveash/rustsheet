# T-1941 W6: lit/dark map of the chain's op sites (coordinator / W4's K1 test). For a base chain, every x87 op site
# (ch.Pj = fl(S_{j+1} x), ch.Sj = fl(P_j + c_j)) has a rounding error e_s(x) if it is stored. A stored site's error
# reaches R as d ln R = +x^j e_s / N (N sites) or -x^j e_s / D (D sites). Joint LS of the data's relative residual
# vs the exact rational of the base chain on all site regressors (per run: mean and linear trend removed, which also
# absorbs smooth coefficient errors): beta ~ 1 with large Z = stored (lit); beta ~ 0 = kept in 80 bits (dark). Run it on
# Excel and on synthetics from the same chain (all stored, or with KEEP masks) to read the signatures.
# Usage: sitemap.py RFILE CHAIN [npts]   env KEEP (unstored sites), PERRUN
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain, nrm
from siteruns import load_runs, Detrend

rf, chain = sys.argv[1], sys.argv[2]
npts = int(sys.argv[3]) if len(sys.argv) > 3 else 400000
N, D = read_chain(chain)
N = [v / nrm(D) for v in N]
D = [v / nrm(D) for v in D]
x, lo, hi, gid = load_runs(rf, npts)
detrend = Detrend(x, gid)
XL = x.astype(L)


KEEP = set(os.environ.get("KEEP", "").split(",")) - {""}  # sites modelled unstored (80-bit, no regressor)


def sites(C, ch):
    """stored-Horner site errors (long double, absolute) and the x^j weights"""
    out = {}
    s = L(C[-1]) * np.ones_like(XL)
    for j in range(len(C) - 2, -1, -1):
        p = s * XL
        ps = p if "%s.P%d" % (ch, j) in KEEP else L(st(p))
        if ps is not p:
            out["%s.P%d" % (ch, j)] = (ps - p) * XL**j
        sv = ps + L(C[j])
        ss = sv if "%s.S%d" % (ch, j) in KEEP else L(st(sv))
        if ss is not sv:
            out["%s.S%d" % (ch, j)] = (ss - sv) * XL**j
        s = ss
    return out, np.polyval(np.array(C, dtype=L)[::-1], XL)


sN, Nv = sites(N, "N")
sD, Dv = sites(D, "D")
R = Nv / Dv
Rm = (lo.astype(L) + hi.astype(L)) / 2
y = np.array((Rm - R) / R, dtype=float)
cols, names = [], []
for nm, e in sN.items():
    cols.append(np.array(e / Nv, dtype=float))
    names.append(nm)
for nm, e in sD.items():
    cols.append(np.array(-e / Dv, dtype=float))
    names.append(nm)
X = np.array(cols).T



y = detrend(y)
X = np.column_stack([detrend(X[:, i]) for i in range(X.shape[1])])
sc = np.sqrt(np.mean(X**2, 0)) + 1e-300
Z_ = X / sc
b, *_ = np.linalg.lstsq(Z_, y, rcond=None)
res = y - Z_ @ b
s2 = np.mean(res**2)
cov = s2 * np.linalg.pinv(Z_.T @ Z_)
beta = b / sc
se = np.sqrt(np.diag(cov)) / sc
print("points %d runs %d  resid rms %.2e (y rms %.2e)" % (len(x), len(np.unique(gid)), np.sqrt(s2), np.sqrt(np.mean(y**2))))
for nm, bb, ss, c in zip(names, beta, se, sc):
    print("  %-6s beta %+7.3f  se %.3f  Z(beta=1 vs 0) %6.1f  size %.1e  %s" % (nm, bb, ss, bb / ss, c, "LIT" if abs(bb - 1) < 3 * ss and bb / ss > 5 else ("dark" if abs(bb) < 3 * ss and 1 / ss > 5 else "?")))

# T-1941 W6: is a "dark" site really unstored, or is the base chain just off there? Scan a constant pre-rounding offset
# delta (units of the site's ulp) at one site (ch.Sj or ch.Pj), keep everything above it as the base chain, and fit
# the data residual (sitemap.py detrending) on that site's rounding-error regressor at each delta. A stored site
# with a mis-set coefficient shows beta ~ 1 (high Z) at some delta; a truly unstored site never does.
# Usage: sitescan.py RFILE CHAIN SITE [RANGE] [STEP] [npts]   (e.g. N.S4 8 0.0625; S sites in ulps of c_j)   env KEEP
import sys, os, numpy as np

sys.path.insert(0, sys.path[0])
from lib6 import st, L
from synth import read_chain, nrm
from siteruns import load_runs, Detrend

rf, chain, site = sys.argv[1:4]
RG = float(sys.argv[4]) if len(sys.argv) > 4 else 8.0
SP = float(sys.argv[5]) if len(sys.argv) > 5 else 1 / 16
npts = int(sys.argv[6]) if len(sys.argv) > 6 else 100000
N, D = read_chain(chain)
N = [v / nrm(D) for v in N]
D = [v / nrm(D) for v in D]
ch, op, J = site[0], site[2], int(site[3:])
x, lo, hi, gid = load_runs(rf, npts)
detrend = Detrend(x, gid)
XL = x.astype(L)
C = N if ch == "N" else D
s = L(C[-1]) * np.ones_like(XL)
pre = None
KEEP = set(os.environ.get("KEEP", "").split(",")) - {""}  # ops above the site left unstored (80-bit)
for j in range(len(C) - 2, -1, -1):
    p = s * XL
    if op == "P" and j == J:
        pre = p
        break
    ps = p if "%s.P%d" % (ch, j) in KEEP else L(st(p))
    sv = ps + L(C[j])
    if op == "S" and j == J:
        pre = sv
        break
    s = sv if "%s.S%d" % (ch, j) in KEEP else L(st(sv))
Nv = np.polyval(np.array(N, dtype=L)[::-1], XL)
Dv = np.polyval(np.array(D, dtype=L)[::-1], XL)
R = Nv / Dv
y = np.array(((lo.astype(L) + hi.astype(L)) / 2 - R) / R, dtype=float)
w = np.array(XL**J / (Nv if ch == "N" else -Dv), dtype=float)
u = np.spacing(np.abs(st(pre))).astype(L)
# S sites: scan in ulps of the coefficient (a constant shift across binades of the site); P sites: in site ulps
if op == "S" and os.environ.get("UNIT") != "site":  # env UNIT=site: scan S sites in site ulps too
    u = L(np.spacing(abs(C[J]))) * np.ones_like(XL)
y = detrend(y)
out = []
for d in np.arange(-RG, RG + 1e-12, SP):
    vv = pre + L(d) * u
    e = detrend(np.array((L(st(vv)) - vv), dtype=float) * w)
    ee = np.dot(e, e)
    b = np.dot(e, y) / ee
    r = y - b * e
    se = np.sqrt(np.mean(r**2) / ee)
    out.append((b / se, b, d))
out.sort(reverse=True)
print("site %s: top deltas (%s):" % (site, "ulps of c%d" % J if op == "S" and os.environ.get("UNIT") != "site" else "ulps of the site"))
for z, b, d in out[:8]:
    print("  delta %+8.4f  beta %+.3f  Z %.1f" % (d, b, z))
z0 = [o for o in out if abs(o[2]) < 1e-12][0]
print("  delta 0: beta %+.3f Z %.1f" % (z0[1], z0[0]))

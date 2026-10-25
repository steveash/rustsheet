"""Does TOMS bfrac (continued fraction with brcomp prefactor; R toms708.c via w2's driver) reproduce Excel's NEGATIVE
response to the rounding of y = fl(1-x) on the a,b > 1 direct runs? Also bup+bpser (R) for comparison."""
import sys
from lib import *
sys.path.insert(0, str(T2464 / "w2-forensics"))
from toms import call
M = mpmath.mpf
R = load(cum=True, branch="direct")
def tie_c(xs, vals, ex):
    d = np.array([float((M(float(sub(1.0, v))) - (1 - M(v))) / (1 - M(v))) / U for v in xs])
    e = rel_units(vals, ex)
    up, dn = e[d > 1e-9], e[d < -1e-9]
    mag = np.abs(d[np.abs(d) > 1e-9]).mean()
    return (up.mean() - dn.mean()) / (2 * mag)
for blk in ("xruns-1:0", "xruns-1:1", "xruns-1:2", "xruns-1:5", "xruns-2:3", "xruns-1:7"):
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = rows[0]["a"]; b = rows[0]["b"]
    E = np.array([r["excel"] for r in rows]); y = sub(1.0, x)
    ex = [exact_cdf(v, a, b) for v in x]
    args = [(a, b, xi, yi) for xi, yi in zip(x, y)]
    for exe in ("toms_sse", "toms_387"):
        bf = np.array(call("bfrac", args, exe=exe))
        br = np.array(call("bratio", args, exe=exe))
        print(f"  {blk} a={a} b={b} {exe}: Excel c {tie_c(x, E, ex):+.2f} | bfrac c {tie_c(x, bf, ex):+.2f} exact {int((bf == E).sum())} | bratio c {tie_c(x, br, ex):+.2f} exact {int((br == E).sum())}")
print("-- complement forms on the direct side: 1 - bfrac(b,a,y,x), 1 - bratio(b,a,y,x)")
for blk in ("xruns-1:0", "xruns-1:1", "xruns-1:2", "xruns-1:5", "xruns-2:3", "xruns-1:7"):
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = rows[0]["a"]; b = rows[0]["b"]
    E = np.array([r["excel"] for r in rows]); y = sub(1.0, x)
    ex = [exact_cdf(v, a, b) for v in x]
    args = [(b, a, yi, xi) for xi, yi in zip(x, y)]
    for exe in ("toms_387",):
        bf = add(sub(0.5, np.array(call("bfrac", args, exe=exe))), 0.5)
        br = add(sub(0.5, np.array(call("bratio", args, exe=exe))), 0.5)
        print(f"  {blk}: Excel c {tie_c(x, E, ex):+.2f} | 1-bfrac(b,a,y,x) c {tie_c(x, bf, ex):+.2f} exact {int((bf == E).sum())} | 1-bratio(b,a,y,x) c {tie_c(x, br, ex):+.2f} exact {int((br == E).sum())}")
print("-- x recomputed from y: bratio(a, b, 1 - y, y), y = fl(1 - x)  (x' = 1 - y is exact)")
for blk in ("xruns-1:0", "xruns-1:1", "xruns-1:2", "xruns-1:5", "xruns-2:3", "xruns-1:7", "xruns-1:3", "xruns-1:6"):
    rows = [r for r in R if r["block"] == blk]
    x = np.array([r["x"] for r in rows]); a = rows[0]["a"]; b = rows[0]["b"]
    E = np.array([r["excel"] for r in rows]); y = sub(1.0, x); xp = sub(1.0, y)
    ex = [exact_cdf(v, a, b) for v in x]
    for exe in ("toms_387", "toms_sse"):
        br = np.array(call("bratio", [(a, b, u, v) for u, v in zip(xp, y)], exe=exe))
        print(f"  {blk} {exe}: Excel c {tie_c(x, E, ex):+.2f} | bratio(a,b,1-y,y) c {tie_c(x, br, ex):+.2f} exact {int((br == E).sum())}")

"""T-3567 r5-fdf1 c1: r2-tdist FINAL's kernel on F at d1 = 1 under (x, y) orientations."""
import math, sys
from collections import Counter
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist")); sys.path.insert(0, str(HERE.parent / "r3-bgrat"))
import xlk, branches  # noqa
sys.argv = sys.argv[:1]
import c0  # noqa: data rows (prints c0's table first)
rows = c0.rows
def xs(x, n):
    s = n + x
    return x / s, n / s
VARS = {
    "x=xx, y=fl(n/s)": lambda x, n: xs(x, n),
    "x=xx, y=fl(1-xx)": lambda x, n: (xs(x, n)[0], 1 - xs(x, n)[0]),
    "x=fl(1-y), y=fl(n/s)": lambda x, n: (1 - xs(x, n)[1], xs(x, n)[1]),
}
for bg in ("ideal", "ld"):
    cfg = xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg=bg)
    for name, f in VARS.items():
        pts = []
        for fn, x, n, up, e in rows:
            lo, hi = f(x, n)
            pts.append((0.5, n / 2, lo, hi))
        out = xlk.run(pts, cfg)
        ex = w4 = 0; br = Counter(); brx = Counter()
        for (fn, x, n, up, e), (w, w1, b) in zip(rows, out):
            v = w1 if up else w
            br[b] += 1
            if v == e: ex += 1; brx[b] += 1
            w4 += abs(v - e) <= 4 * math.ulp(e)
        print(f"bg={bg:5s} {name:22s} exact {ex}/{len(rows)} w4 {w4}  " + ", ".join(f"{k}:{brx[k]}/{br[k]}" for k in br))

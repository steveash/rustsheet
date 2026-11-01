"""T-3567 r5-fdf1 c5: does the r >= 1 (x, y) rule hold for F at every d1? FINAL's kernel on all F cdf/tail cells."""
import csv, math, sys
from collections import defaultdict
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa
CFG = xlk.XCfg(alnrel="xl", erfc="xl", bg_lnx="cut0.01", bg="ld")
rows = []
for r in csv.DictReader(open(HERE.parent / "cells.tsv"), delimiter="\t"):
    if r["fn"] not in ("FDIST", "F.DIST.RT", "F.DIST"): continue
    p = r["args"].split("|")
    if r["fn"] == "F.DIST" and p[3] in ("False", "0.0"): continue
    try:
        x, m, n, e = float(p[0]), math.trunc(float(p[1])), math.trunc(float(p[2])), float(r["excel"])
    except ValueError:
        continue
    if x <= 0 or m < 1 or n < 1 or e == 0 or not math.isfinite(m * x): continue
    rows.append((x, m, n, r["fn"] != "F.DIST", e))
def s_form(x, m, n):
    mx = m * x; s = n + mx; return mx / s, n / s
def rbig(x, m, n):
    mx = m * x
    if mx <= n:
        r = n / mx; return 1 / (1 + r), r / (1 + r)
    r = mx / n; return r / (1 + r), 1 / (1 + r)
def rdens(x, m, n):
    mx = m * x
    if mx <= n:
        r = mx / n; return r / (1 + r), 1 / (1 + r)
    r = n / mx; return 1 / (1 + r), r / (1 + r)
def eng_form(x, m, n):       # fisher.rs f_beta_arguments today: both as 1/(1 + .)
    mx = m * x
    return 1 / (1 + n / mx), 1 / (1 + mx / n)
print(len(rows), "F cdf/tail cells")
for name, f in (("s=n+mx", s_form), ("r>=1", rbig), ("r<=1 (density sides)", rdens), ("both 1/(1+.) (engine)", eng_form)):
    pts = [(m / 2, n / 2, *f(x, m, n)) for x, m, n, up, e in rows]
    out = xlk.run(pts, CFG)
    tab = defaultdict(lambda: [0, 0, 0])
    for (x, m, n, up, e), (w, w1, b) in zip(rows, out):
        v = w1 if up else w
        for k in ("all", b.split("/")[0], "d1=1" if m == 1 else ("d2=1" if n == 1 else "d1,d2>1")):
            t = tab[k]; t[0] += 1; t[1] += v == e; t[2] += abs(v - e) <= 4 * math.ulp(e)
    print(f"{name:22s} " + "; ".join(f"{k} {t[1]}/{t[2]}/{t[0]}" for k, t in sorted(tab.items())))

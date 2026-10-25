"""Out-of-sample check of the b = 1/2 bgrat identity on non-T cells: r2-bratio's data sets (BETA.DIST, F, BINOM,
x29 probe), rows whose BEST branch involves bgrat with min(a, b) = 1/2. The caller's x, y are those r2-bratio uses
(BETA.DIST: y = fl(1 - x); F: w5's forms). Scores BEST, BEST + Excel log1p/ERFC, and the ideal bgrat (cut 0.01).
usage: oos_bgrat.py"""
from collections import defaultdict

import tlib
import tscore
import xlk

sl = tlib.sl
rows = [r for r in sl.load() if not r["blk"].startswith("T.") and r["blk"] != "TDIST"]
SPECS = {
    "BEST": "",
    "xl": "alnrel=xl,erfc=xl",
    "xl+cut": "alnrel=xl,erfc=xl,bg_lnx=cut0.01",
    "ideal": "alnrel=xl,erfc=xl,bg_lnx=cut0.01,bg=ideal",
    "ideal_toms_lnx": "alnrel=xl,erfc=xl,bg=ideal",
}
pts = [(r["a"], r["b"], r["x"], r["y"]) for r in rows]
base = xlk.run(pts, tscore.parse(SPECS["BEST"])[1])
keep = [i for i, v in enumerate(base) if "bgrat" in v[2] and min(rows[i]["a"], rows[i]["b"]) == 0.5]
rows = [rows[i] for i in keep]
pts = [pts[i] for i in keep]
br = [base[i][2] for i in keep]
out = defaultdict(list)
for name, spec in SPECS.items():
    res = xlk.run(pts, tscore.parse(spec)[1])
    for r, v, b in zip(rows, res, br):
        m = sl.side_value(r, v)
        for key in ((name, "all"), (name, b), (name, b, r["set"])):
            out[key].append((m, r["excel"]))
for r, b in zip(rows, br):
    for key in (("CR", "all"), ("CR", b), ("CR", b, r["set"])):
        out[key].append((r["cr"], r["excel"]))
xlk.save_cache()
for k in sorted(out, key=lambda k: (k[1:], k[0])):
    print(tlib.line(str(k), out[k]))

"""Positive control 1: the port in R_LIKE configuration (glibc exp/log, TOMS gamln, R's 0.29, R's bgrat/algdiv)
must reproduce R's toms708.c bratio (w2-forensics/ext/bratio_sse, SSE doubles) bit for bit, and sit near the
correctly rounded value. Also TOMS_XL (only Excel's EXP/LN/GAMMALN substituted) for reference."""
import subprocess
import sys
from collections import Counter, defaultdict

import bratio_excel as bx
import score_lib as sl

rows = sl.load()
pts = sorted({(r["a"], r["b"], r["x"], r["y"]) for r in rows})
print("rows", len(rows), "distinct points", len(pts))
exe = sl.T2464 / "w2-forensics" / "ext" / "bratio_sse"
inp = "".join(f"{a.hex()} {b.hex()} {x.hex()} {y.hex()}\n" for a, b, x, y in pts)
out = subprocess.run([str(exe)], input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
R = {p: tuple(float.fromhex(t) for t in line.split()[:2]) for p, line in zip(pts, out)}
port = dict(zip(pts, bx.run(pts, bx.R_LIKE)))
agree = Counter()
bad = defaultdict(list)
for p in pts:
    w, w1, br = port[p]
    ok = (w, w1) == R[p]
    agree[(br.split("/")[0], ok)] += 1
    if not ok:
        bad[br.split("/")[0]].append((p, w, R[p][0], sl.ulpd(w, R[p][0])))
print("\nR_LIKE port vs R bratio binary (distinct points), by branch: bit-identical (w and w1) / total")
for br in sorted({k[0] for k in agree}):
    t, f = agree[(br, True)], agree[(br, False)]
    mx = max((d for *_, d in bad[br]), default=0)
    print(f"  {br:22s} {t:6d}/{t + f:<6d}  max |ulp| of misses {mx:.1f}")
for br, lst in bad.items():
    for item in lst[:3]:
        print("   e.g.", br, item)
# port vs CR (the value each row asks for)
vals = {"R_LIKE": [sl.side_value(r, port[(r["a"], r["b"], r["x"], r["y"])]) for r in rows]}
crd = Counter()
for r, v in zip(rows, vals["R_LIKE"]):
    crd[min(int(sl.ulpd(v, r["cr"])), 9)] += 1
print("\nR_LIKE port vs correctly rounded (rows): |ulp| histogram (9 = 9+):", dict(sorted(crd.items())))

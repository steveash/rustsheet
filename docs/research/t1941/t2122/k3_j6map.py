"""Summarise a k3_j6scan.sh run (T-2122 run 6, HANDOFF2 18.3): the best-b score per (c, d) pair from the parts' .npy,
the base ridge (a line c = k d + m fitted to the top 50) and the best scores on and off it. With TRUTH=c,d (control)
also the score at the grid pair nearest the planted truth and its rank among off-ridge pairs.
usage: k3_j6map.py OUTDIR [MODE]   (MODE real|control, picks scan3_N6_<MODE>*.npy)"""
import glob
import os
import sys

import numpy as np

mode = sys.argv[2] if len(sys.argv) > 2 else "real"
a = np.concatenate([np.load(f) for f in sorted(glob.glob(os.path.join(sys.argv[1], "scan3_N6_%s*.npy" % mode)))])
v, A, B, C, D = a.T
print("pairs %d, median %.1f, sd %.1f, max %.1f" % (len(a), np.median(v), v.std(), v.max()))
o = np.argsort(v)[::-1]
k = np.polyfit(D[o[:50]], C[o[:50]], 1)
print("base ridge (top 50): c = %.3f d %+.2f" % tuple(k))
r = C - (k[0] * D + k[1])
for lab, m in [("on ridge |r| <= 4", np.abs(r) <= 4), ("off ridge |r| > 8", np.abs(r) > 8), ("off ridge |r| > 16", np.abs(r) > 16)]:
    i = np.where(m)[0]
    j = i[np.argsort(v[i])[::-1][:6]]
    print("%s: n %d, median %.1f, sd %.1f; top %s" % (lab, len(i), np.median(v[i]), v[i].std(),
          " | ".join("%.0f (b %.1f c %g d %g)" % (v[x], B[x], C[x], D[x]) for x in j)))
for dd in (-400, -200, -100, -48, -24, -12, 0, 12, 24, 48, 100, 200, 400):
    m = (np.abs(D - dd) <= 2) & (np.abs(r) <= 4)
    if m.any():
        print("  ridge d %+5d: best %.0f" % (dd, v[m].max()))
if os.environ.get("TRUTH"):
    tc, td = map(float, os.environ["TRUTH"].split(","))
    n = np.argmin((C - tc) ** 2 / 4 + (D - td) ** 2 / 16)
    off = np.abs(r) > 8
    print("nearest grid pair to truth (c %g, d %g): %.1f (b %.1f), r %.1f; off-ridge rank %d of %d" % (
        C[n], D[n], v[n], B[n], r[n], int((v[off] > v[n]).sum()), int(off.sum())))

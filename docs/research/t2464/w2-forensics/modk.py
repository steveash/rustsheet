"""Mean of Excel's relative error by k mod 8 (k = mantissa low bits of x) per block."""
import csv, collections, sys, struct
import numpy as np
rows = list(csv.DictReader(open("exact_w2f.tsv"), delimiter="\t"))
case = sys.argv[1]
g = collections.defaultdict(list)
for r in rows:
    if r["case"] != case: continue
    i = int(r["cell"].split("!A")[1]) - 1
    x = float(r["x"]); m = struct.unpack("<Q", struct.pack("<d", x))[0] & 15
    g[(i // 2048, r["cum"])].append((m, float(r["rel"]), x))
for k in sorted(g):
    v = g[k]
    by = collections.defaultdict(list)
    for m, e, x in v: by[m % 8].append(e)
    om = 1 - v[0][2]
    print(k, f"x={v[0][2]:.5g}", " ".join(f"{np.mean(by[j]):+6.2f}" for j in range(8)))

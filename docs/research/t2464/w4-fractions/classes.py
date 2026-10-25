"""Excel direct-cdf error (units 2^-53 rel) by the low 3 mantissa bits of x, per w2 xruns block."""
import csv, collections, struct, mpmath
import numpy as np
rows = list(csv.DictReader(open("../w2-forensics/exact_w2f.tsv"), delimiter="\t"))
g = collections.defaultdict(list)
for r in rows:
    if "xruns" not in r["case"]: continue
    i = int(r["cell"].split("!A")[1]) - 1
    g[(r["case"][-1], i // 2048, r["cum"])].append(r)
def man(x): return struct.unpack('<Q', struct.pack('<d', x))[0] & ((1<<52)-1)
for k in sorted(g):
    rs = g[k]; a, b = float(rs[0]["a"]), float(rs[0]["b"])
    cls = collections.defaultdict(list); dl = collections.defaultdict(list)
    for r in rs:
        x = float(r["x"]); om = 1 - x
        d = float((mpmath.mpf(om) - (1 - mpmath.mpf(x))) / (1 - mpmath.mpf(x))) * 2**53
        cls[man(x) & 7].append(float(r["rel"])); dl[man(x)&7].append(d)
    print(k, a, b, rs[0]["x"][:6], ' '.join(f"{c}:{np.mean(cls[c]):+5.2f}[{np.mean(dl[c]):+.2f}]" for c in range(8)))

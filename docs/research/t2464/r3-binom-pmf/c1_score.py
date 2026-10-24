"""Score the frozen models (blib.MODELS + CR) on a data set: exact / within 4 ulp / within 15 digits per fn and
region (k = 0, k = n, interior).  Usage: c1_score.py <cells|a|n|hold> [models comma]"""
import collections
import sys

import blib as B

src = sys.argv[1]
models = (sys.argv[2] if len(sys.argv) > 2 else "CR," + ",".join(B.MODELS)).split(",")
rows = B.load_cells() if src == "cells" else B.load_probe(src)
rows = [r for r in rows if r["n"] <= 10**7 and r["p"] > 1e-6 and (r["fn"] != "NEG" or r["n"] >= 1)]


def region(r):
    if r["fn"] == "NEG":
        return "NEG"
    kk = r["k"]
    return f"{r['fn']} k=0" if kk == 0 else f"{r['fn']} k=n" if kk == r["n"] else f"{r['fn']} interior"


for m in models:
    g = collections.defaultdict(list)
    for r in rows:
        if r["fn"] == "NEG" and src == "cells":  # cells.tsv NEG rows: f = k, r = n -> shifted kernel (n-1, k+n-1)
            k, n = r["n"], r["k"] + r["n"]
        else:
            k, n = r["k"], r["n"]
        try:
            v = B.model_value(m, r["fn"], k, n, r["p"])
        except (ValueError, ZeroDivisionError, OverflowError):
            v = None
        g[region(r)].append((v, r["excel"]))
        g["all"].append((v, r["excel"]))
    print(f"{m:12s} " + "  ".join(f"{gname}: {B.fmt(B.tally(v))}" for gname, v in sorted(g.items())))

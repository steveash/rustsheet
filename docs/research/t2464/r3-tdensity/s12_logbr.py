"""In-sample a1/a2: free-constant scan of Loader-type log-branch models (x x > 0.2 n) per df, shared by all
log-branch blocks of that df.  usage: s12_logbr.py '<json opts>' ...   (C0 = exact Loader constant [- ln sqrt(2 pi)
for den sw/swm/sy])"""
import json
import sys
from collections import defaultdict


import tdlib as TD

L = TD.LDB
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
SER = "--series" in sys.argv
rows = [r for r in rows if (r["t"] * r["t"] > 0.2 * r["n"]) != SER or "--all" in sys.argv]
BLK = [a[6:] for a in sys.argv if a.startswith("--blk=")]
rows = [r for r in rows if not BLK or any(b in r["blk"] for b in BLK)]
by = defaultdict(list)
for r in rows:
    by[r["n"]].append(r)
for arg in [a for a in sys.argv[1:] if not a.startswith("--")]:
    o = json.loads(arg)
    tot = [0, 0]
    out = []
    for n in sorted(by):
        rr = by[n]
        nb, pos, ivs, _ = TD.fit_free_C(rr, n, o)
        per = defaultdict(lambda: [0, 0])
        for iv, r in zip(ivs, rr):
            per[r["blk"]][1] += 1
            per[r["blk"]][0] += any(lo <= pos <= hi for lo, hi in iv)
        tot[0] += nb
        tot[1] += len(rr)
        out.append(f"n{n:g} {nb}/{len(rr)}@{pos:+.2f}[" + " ".join(f"{v[0]}/{v[1]}" for b, v in sorted(per.items())) + "]")
    print(f"{arg}: {tot[0]}/{tot[1]} | " + "; ".join(out), flush=True)

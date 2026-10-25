"""Free-constant scan of Loader's dt (old R form) with x87 knobs, per df, on the exact-input runs
(t0969-t-df-ratio, t*t exact) and the pdf-runs.  usage: s1_loader.py <variant-json> [maxcells/df]"""
import json
import sys
from collections import defaultdict

import tdlib as TD

o = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
mx = int(sys.argv[2]) if len(sys.argv) > 2 else 400
rows = TD.load_old()
grp = defaultdict(list)
for r in rows:
    if r["n"] >= 3 and ("df-ratio" in r["case"] or "pdf-runs" in r["case"]):
        grp[(r["case"].split("/")[1][6:16], r["n"])].append(r)
tot = [0, 0]
for key in sorted(grp):
    rr = grp[key][:mx]
    C0 = TD.ld_of(TD.loader_const_exact(key[1]))
    ivs = []
    for r in rr:
        F = lambda u, r=r: TD.loader_dt(r["t"], r["n"], C0 + TD.LDB(u) * TD.U53, o)  # noqa: E731
        ivs.append(TD.interval(F, r["excel"]))
    nb, pos = TD.best_overlap(ivs)
    h0 = TD.hits_at(ivs, 0.0)
    tot[0] += nb
    tot[1] += len(rr)
    print(f"{key}: best {nb}/{len(rr)} at C0{pos:+.3f}u; at exact C {h0}", flush=True)
print("total", tot)

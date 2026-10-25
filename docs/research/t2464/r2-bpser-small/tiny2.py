"""Probe tiny2: tiny-literal parse errors, and apser/fpser with exact arguments vs the literal ones (probe tiny)."""
import json
from collections import Counter

import lib

meta = json.load(open(lib.HERE / "t2464-r2-bpser-small-tiny2.meta.json"))
out = json.load(open(lib.HERE / "t2464-r2-bpser-small-tiny2.out.json"))["sheets"]["Calc"]
old = json.load(open(lib.HERE / "t2464-r2-bpser-small-tiny.out.json"))["sheets"]["Calc"]
par = [(b, out[i]["v"]) for i, blk, b, _, _ in meta if blk == "parse"]
print("tiny literals parsed exactly:", sum(v == 0 for _, v in par), "of", len(par))
print("relative parse errors:", sorted(Counter(round(v / 2.220446049250313e-16, 2) for _, v in par).items())[:12])
rows = [r for r in lib.load_probe(("tiny",))]
byi = {r["i"]: r for r in rows} if rows and "i" in rows[0] else None
pts, ex, ex_old, cr, labs = [], [], [], [], []
cache = {}
for i, blk, x, a, b in meta:
    if blk == "parse":
        continue
    pts.append((a, b, x, 1.0 - x))
    ex.append(out[f"A{i}"]["v"])
    ex_old.append(old[f"A{i}"]["v"])
    cr.append(lib.sl._exact_cdf(x, a, b, cache))
res = lib.run(pts)
c = Counter()
for p, e, eo, q, r in zip(pts, ex, ex_old, cr, res):
    br = lib.label(r)
    c[(br, "n")] += 1
    c[(br, "exact-args == literal-args")] += e == eo
    for nm, m in (("model", r[0]), ("CR", q)):
        t = lib.tally([(m, e)])
        c[(br, nm + " exact")] += t[1]
        c[(br, nm + " w4")] += t[2]
        c[(br, nm + " w15")] += t[3]
for k in sorted(c):
    print(k, c[k])

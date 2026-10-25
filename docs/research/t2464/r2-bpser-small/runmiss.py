"""Per run (probe runs): exact rate of a variant, signed error histogram, and the miss rate against features of each
cell (binade of s, of E = EXP(z), parity of the series length, the E*s product's tie distance).
usage: runmiss.py [variant=L53] [run=substring]"""
import sys
from collections import Counter, defaultdict

import lib
import bscore as score

kw = dict(a.split("=", 1) for a in sys.argv[1:] if "=" in a)
v = score.VARIANTS[kw.get("variant", "L53")]
rows = [r for r in lib.load_probe(("runs",)) if kw.get("run", "") in r["blk"]]
res = lib.run([(r["a"], r["b"], r["x"], r["y"]) for r in rows], v)
m = lib.Model(lib.bx.BEST, v)
byrun = defaultdict(list)
for r, q in zip(rows, res):
    byrun[r["blk"]].append((r, q))
for blk, items in byrun.items():
    errs = Counter()
    feat = defaultdict(lambda: [0, 0])
    for r, q in items:
        val = q[0]
        d = lib.ulps_signed(val, r["excel"])
        errs[round(d)] += 1
        fn, a, b, x = q[3][-1] if q[3] else ("", 0, 0, 0)
        if fn == "bpser" and q[2] == "w_bpser":
            s = m.series_w3(a, b, x)
            e = m.w3_pre(a, b, x)
            p = float(lib.mpf(e) * s)
            for k, f in (("s binade", lib.math.frexp(s)[1]), ("E binade", lib.math.frexp(e)[1]),
                         ("I binade", lib.math.frexp(r["excel"])[1]),
                         ("E*s frac ulp", round(((lib.mpf(e) * s - lib.mpf(p)) / lib.math.ulp(p)) * 4) / 4)):
                feat[(k, f)][0] += 1
                feat[(k, f)][1] += d == 0
    print(blk, f"exact {errs[0]}/{len(items)}", "errors:", dict(sorted(errs.items())))
    for k in sorted(feat):
        n, ex = feat[k]
        if n >= 8:
            print(f"    {k}: {ex}/{n}")

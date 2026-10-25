"""Print Excel's error (units 2^-53 rel) along a consecutive-double run, with x's low mantissa bits and ulp(excel)."""
import math
import sys
import dlib as L

rows = L.load()
blk = sys.argv[1]
n = int(sys.argv[2]) if len(sys.argv) > 2 else 64
idx = sorted(L.blocks(rows)[blk], key=lambda i: rows[i]["x"])
for i in idx[:n]:
    r = rows[i]
    m = int(math.frexp(r["x"])[0] * 2**53)
    e = L.err_units(r["excel"], r["exact"])
    ecr = L.err_units(float(r["exact"]), r["exact"])
    print(f"{r['x']!r:24s} low3={m & 7} err {e:+6.2f} crerr {ecr:+6.2f} ulp_units {math.ulp(r['excel'])/r['excel']*2**53:.2f}")

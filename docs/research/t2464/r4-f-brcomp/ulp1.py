"""Misses of the frozen M-xd at +-1 ulp (lesson: check misses at one ulp), per set."""
import math
import sys

import fblib as B
import ratlib as RL

O = dict(pw="xd", hl="ext", grp="none", est="ext", L="xl")
for s in sys.argv[1:]:
    rows = B.load(s)
    miss = one = 0
    for r in rows:
        v = RL.value(r["x"], r["d1"], r["d2"], O)
        if v != r["excel"]:
            miss += 1
            one += v in (math.nextafter(r["excel"], 0), math.nextafter(r["excel"], math.inf))
    print(f"{s}: {miss} misses, {one} of them 1 ulp off")

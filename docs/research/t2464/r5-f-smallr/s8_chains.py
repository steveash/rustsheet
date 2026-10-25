"""Score chains (r5lib chain grammar) per run with a free constant.  Usage: s8_chains.py SET tagfilter chain [chain ...]
('all' as tagfilter: every small-r informative run)."""
import sys

import r5lib as CL
import r5lib as R

src, tag = sys.argv[1], sys.argv[2]
chains = sys.argv[3:]
tot = [0] * len(chains)
N = 0
pred = R.small_r if tag == "all" else (lambda m, n, rs: tag in rs[0]["blk"])
for blk, rs, m, n in R.runs(src, pred):
    sc = [CL.scan(rs, m, n, c)[0] for c in chains]
    N += len(rs)
    tot = [t + s for t, s in zip(tot, sc)]
    print(f"{blk:28s} n={len(rs)}", " ".join(f"{s:4d}" for s in sc))
    sys.stdout.flush()
print("TOTAL of", N)
for c, t in sorted(zip(chains, tot), key=lambda z: -z[1]):
    print(f"  {c:30s} {t}")

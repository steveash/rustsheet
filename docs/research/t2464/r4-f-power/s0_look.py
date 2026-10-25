"""s0: look at D along large-shape runs of r3 probe a."""
import collections
import numpy as np
import plib as P

rows = P.r3_rows("a")
g = collections.defaultdict(list)
for r in rows:
    g[r["blk"]].append(r)
for blk, rs in g.items():
    m, n = rs[0]["d1"], rs[0]["d2"]
    if max(m, n) < 50:
        continue
    D = np.array([P.D_units(r) for r in rs])
    print(blk, f"sd {D.std():.1f}", " ".join(f"{d:.0f}" for d in D[:48]))

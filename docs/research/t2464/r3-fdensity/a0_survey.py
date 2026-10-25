"""a0: CR scores and error spreads of Excel's F density on cells.tsv and r3-tdensity's d1 = 1 column, by df class."""
import collections
import statistics as st

import flib as F

rows = F.with_exact(F.cells_rows() + F.tdens_rows())
g = collections.defaultdict(list)
for r in rows:
    c = "d1=1" if r["d1"] == 1 else ("d1=2" if r["d1"] == 2 else ("d2<=2" if r["d2"] <= 2 else "d1,d2>2"))
    g[(r["src"].split("-")[0], c)].append(r)
for k, v in sorted(g.items()):
    us = [F.units(r["cr"], r["excel"]) for r in v]
    print(k, F.fmt(F.tally([(r["cr"], r["excel"]) for r in v])), "CR-excel mean %.2f sd %.2f" % (st.mean(us), st.pstdev(us)))

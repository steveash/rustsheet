"""M-xd vs CR on a set split by the L branch (r <= 0.375 series, r > 0.375 LN(fl(1+r))). Usage: split.py SET"""
import sys

import vlib as V

rows, _ = V.case_rows(sys.argv[1])
md = V.mxd(rows)
for lab, keep in (("r<=0.375", lambda rr: rr <= 0.375), ("r>0.375", lambda rr: rr > 0.375)):
    idx = [i for i, r in enumerate(rows) if keep(V._pre(r["x"], r["d1"], r["d2"])[2])]
    cr = [(float(V.f_exact(rows[i]["x"], rows[i]["d1"], rows[i]["d2"])), rows[i]["excel"]) for i in idx]
    mm = [(md[i], rows[i]["excel"]) for i in idx]
    print(f"{sys.argv[1]} {lab}: CR {V.fmt(V.tally(cr))}  M-xd {V.fmt(V.tally(mm))}")

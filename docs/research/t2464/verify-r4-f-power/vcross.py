"""Cell-by-cell agreement of this directory's dd/ddl/m1 (vlib.model) with r4-f-power's fmodel.value, after reading
fmodel.py against its description.  Usage: vcross.py SRC"""
import os
import sys

import vlib as V
import vscore as S

R4 = V.T / "r4-f-power"
sys.path[:] = [p for p in sys.path if not p.endswith("r2-log1p")]  # its make_probes/cands shadow r4's
sys.path.insert(0, str(R4))
os.chdir(R4)
import fmodel  # noqa: E402

rs = S.rows(sys.argv[1])
S.prebatch(rs)
for var in ("m1", "dd", "ddl"):
    diff = sum(V.model(r["x"], r["d1"], r["d2"], var) != fmodel.value(r["x"], r["d1"], r["d2"], var) for r in rs)
    print(f"{sys.argv[1]} {var}: {diff} of {len(rs)} cells differ from fmodel.value")

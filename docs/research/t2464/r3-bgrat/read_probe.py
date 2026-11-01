"""T-2512 r3-bgrat: probe t2512-r3-bgrat-qgamma -> qxl.pkl (git-ignored): {(src, b, z): Excel's Q(b, z)}.
src: "chirt" = CHISQ.DIST.RT(2z, 2b) (Excel's direct Q); "1-P" = 1 - GAMMA.DIST(z, b, 1, TRUE), exact difference."""
import json
import pickle
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
NAME = "t2512-r3-bgrat-qgamma"
CASES = HERE.parents[3] / "eval" / "suites" / "probes-numeric" / "cases"
out = json.load(open(CASES / f"{NAME}.oracle.json"))["sheets"]["Calc"]
q = {}
for i, b, z in json.load(open(HERE / f"{NAME}.meta.json")):
    a, c = out.get(f"A{i}"), out.get(f"B{i}")
    if c and c.get("t") == "n":
        q[("chirt", b, z)] = c["v"]
    if a and a.get("t") == "n":
        q[("1-P", b, z)] = 1 - mpmath.mpf(a["v"])
pickle.dump(q, open(HERE / "qxl.pkl", "wb"))
print(len(q))

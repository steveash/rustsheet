"""Frozen-model scores (bit-exact / within 4 ulp / within 15 digits, by class) on in-sample and hold-out sets.
Models (frozen before hold4 was read):
  CR          correctly rounded exact density
  brcomp      TOMS 708 brcomp(a, b, y, q)/x at w5's (y, q), bratio's swap, Excel EXP/LN, TOMS gamln (section 3)
  r3-ratio    r3-fdensity c1's ratio-xl-e (imported)
  M-xd        ratlib pw=xd, hl=ext, grp=none, est=ext, L=xl: E = K + fl(c LN x) - h L(r), value / x
  M-x         ratlib pw=x (fl((a-1) LN x) / fl(-(b+1) LN x)), otherwise as M-xd
Usage: final_score.py SET [SET ...]   (SET: a sw cells big | hold4 hold holdsw)"""
import sys
from pathlib import Path

import fblib as B
import ratlib as RL

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r3-fdensity"))
import c1_model as C1  # noqa: E402  (r3-fdensity: the ratio model)

MODELS = {
    "CR": lambda r: r["cr"],
    "brcomp": lambda r: B.model(r["x"], r["d1"], r["d2"], {"gamln": "toms"}),
    "r3-ratio": lambda r: C1.value(r["x"], r["d1"], r["d2"], C1.VARIANTS["ratio-xl-e"]),
    "M-xd": lambda r: RL.value(r["x"], r["d1"], r["d2"], dict(pw="xd", hl="ext", grp="none", est="ext", L="xl")),
    "M-x": lambda r: RL.value(r["x"], r["d1"], r["d2"], dict(pw="x", hl="ext", grp="none", est="ext", L="xl")),
}
for s in sys.argv[1:]:
    rows = B.load(s)
    print(f"# {s}: {len(rows)} rows")
    for k, fn in MODELS.items():
        print(B.line(k, B.score(rows, lambda r, fn=fn: float(fn(r)))))

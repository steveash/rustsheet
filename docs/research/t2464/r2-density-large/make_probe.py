"""T-2464 r2-density-large probe 1: BETA.DIST(x,a,b,FALSE) only, consecutive-double runs.

Blocks (each a run of consecutive doubles from an irregular start):
  cut:    (2.5,3.5) runs of 256 placed so the p side of Loader's bd0 has e = (n x - k)/k at a chosen value (k = 1.5,
          n = 4, so n x is exact): e in {-0.40,-0.38,-0.36,-0.34,-0.30,+0.30,+0.34,+0.38,+0.40,+0.45,+0.50,+0.55};
          separates a ratio cut |d| < 0.2 (k + np) from an |e| <= 0.375 cut.
  str8:   a in {7.5, 8, 8.5} x b in {8, 12, 40}, 512 near the mode each (the TOMS a0 >= 8 boundary).
  shp:    16 shapes with a, b > 1 (incl. mirrors of probed shapes), 256 near the mode each (shape constants).
  hold:   (2.5,3.5) and (7.3,11.6), 1024 each at irregular x away from the earlier runs (hold-out).
Writes eval/suites/probes-numeric/cases/t2464-r2-density-large-p1.case.json and probe_p1.meta.json here.
Formula text reuses w2-forensics' exact-double encoder (make_probes.num / run / case).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).parent
sys.path.append(str(HERE.parent / "w2-forensics"))
import make_probes as MP  # noqa: E402

NAME = "t2464-r2-density-large-p1"


def blocks():
    out = []  # (blk, a, b, x0, n)
    for e in (-0.40, -0.38, -0.36, -0.34, -0.30, 0.30, 0.34, 0.38, 0.40, 0.45, 0.50, 0.55):
        out.append((f"cut{e:+.2f}", 2.5, 3.5, 0.375 * (1 + e) + 1.37e-7, 256))
    for a in (7.5, 8.0, 8.5):
        for b in (8.0, 12.0, 40.0):
            mode = (a - 1) / (a + b - 2)
            out.append((f"s8({a:g},{b:g})", a, b, mode * 1.0213 + 0.00071, 512))
    shapes = [
        (1.5, 1.5), (1.25, 5.0), (3.0, 2.0), (6.0, 1.5),
        (2.0, 2.0), (3.0, 3.0), (2.5, 2.5), (5.0, 9.0),
        (1.1, 1.9), (1.9, 3.3), (4.5, 4.5), (3.5, 2.5),
        (10.0, 10.0), (20.0, 30.0), (1.03, 40.0), (11.6, 7.3),
    ]
    for a, b in shapes:
        mode = (a - 1) / (a + b - 2)
        out.append((f"shp({a:g},{b:g})", a, b, mode * 0.9871 + 0.00113, 256))
    out.append(("hold(2.5,3.5)", 2.5, 3.5, 0.5179311, 1024))
    out.append(("hold(7.3,11.6)", 7.3, 11.6, 0.3517713, 1024))
    return out


def build():
    rows, meta = [], []
    for blk, a, b, x0, n in blocks():
        for x in MP.run(x0, 0, n):
            rows.append((x, a, b, False))
            meta.append(["Calc", f"A{len(rows)}", blk, x, a, b])
    c = MP.case(
        NAME,
        "T-2464 r2-density-large: BETA.DIST pdf runs for a, b > 1 (bd0 cut, a0 = 8 straddle, shape "
        "constants, hold-out)",
        rows,
    )
    return c, meta


if __name__ == "__main__":
    c, meta = build()
    print(NAME, len(meta))
    if "--write" in sys.argv:
        p = MP.CASES / f"{NAME}.case.json"
        assert not p.exists()
        p.write_text(json.dumps(c, indent=1) + "\n")
        (HERE / "probe_p1.meta.json").write_text(json.dumps({"case": NAME, "rows": meta}))

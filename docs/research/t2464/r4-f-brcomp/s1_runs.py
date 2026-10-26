"""Per-run diagnosis: offset (median units, model - Excel) and the within-run sd after removing it, model vs CR.
A model with the right x-structure has a small within-run sd even when its constant is off."""
import statistics as S
import sys

import fblib as B

SET = sys.argv[1] if len(sys.argv) > 1 else "a"
VARIANTS = {
    "toms-gl": {},
    "toms-tg": {"gamln": "toms"},
    "xlaln-tg": {"gamln": "toms", "alnrel": "xl"},
    "rlogx-tg": {"gamln": "toms", "rlog1": "exact"},
}
rows = B.load(SET)
B.prefetch_gl(rows, VARIANTS.values())
runs: dict = {}
for r in rows:
    runs.setdefault(r["blk"], []).append(r)
print(f"# {SET}: per run, offset / within-run sd in units (model - Excel)")
print(f"{'run':24s} {'CR':>12s} " + " ".join(f"{k:>14s}" for k in VARIANTS))
for blk, rr in runs.items():
    cols = [[B.F.units(r["cr"], r["excel"]) for r in rr]]
    for o in VARIANTS.values():
        cols.append([B.F.units(B.model(r["x"], r["d1"], r["d2"], o), r["excel"]) for r in rr])
    out = []
    for c in cols:
        c = [v for v in c if v == v]
        out.append(f"{S.median(c):+7.1f}/{S.pstdev(c):5.1f}" if len(c) > 1 else "-")
    print(f"{blk:24s} " + " ".join(f"{v:>14s}" for v in out))

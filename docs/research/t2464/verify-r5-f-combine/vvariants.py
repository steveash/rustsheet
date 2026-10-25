"""verify-r5-f-combine: the EXP-storage confound. M-sd (sdmodel) stores EXP's result as a double before the 1/x step;
hkp-sd (final_score) keeps it extended. Score each model with both, plus M-sd with E rounded once.
Usage: vvariants.py CASE_ID..."""

from __future__ import annotations

import sys

import vmodel as VM
import vparse as VP
import vtally as VT

VARS = ("M-sd", "M-sd+xexp", "M-sd1", "M-sdm", "M-sdm+xexp", "M-sdm1", "hkp-sd", "hkp-sd+dexp")

for c in sys.argv[1:]:
    rows = [r for r in VP.rows(c) if VM.side_r(r["x"], r["d1"], r["d2"])[1] > 0.375]
    VM.prefetch_ln([v for r in rows for v in VM.ln_inputs(r["x"], r["d1"], r["d2"])])
    print(f"{c} (r > 0.375): {len(rows)} cells")
    for mdl in VARS:
        print(f"  {mdl:12s} {VT.fmt(VT.tally((VM.value(r['x'], r['d1'], r['d2'], mdl), r['excel']) for r in rows))}")
        sys.stdout.flush()

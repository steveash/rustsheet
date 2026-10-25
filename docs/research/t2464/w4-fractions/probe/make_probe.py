"""Build probe/w4p1.case.json: for the a,b>1 direct-branch runs of w2-forensics (xruns-1:0, 1:1, 1:2, 1:5, 2:3), the
two pieces TOMS 708's L140 path would use: the density BETA.DIST(1-x, b0', a, FALSE) (bup's brcmp1 = pdf*y0*x0) and
BETA.DIST(x, a, b0', TRUE) (bpser), plus 1-x itself.  b0' = b - int(b) in double (1 when b is an integer)."""
import csv, json, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
rows = list(csv.DictReader(open(HERE.parent / 'data.tsv'), delimiter='\t'))
BLKS = ['xruns-1:0', 'xruns-1:1', 'xruns-1:2', 'xruns-1:5', 'xruns-2:3']
sheets = []; meta = []
for j, blk in enumerate(BLKS):
    rs = [r for r in rows if r['blk'] == blk and r['branch'] == 'direct']
    cells = {}
    for k, r in enumerate(rs):
        x, a, b = float(r['x']), float(r['a']), float(r['b'])
        n = int(b); b0 = b - n
        if b0 == 0: b0 = 1.0
        i = k + 1
        cells[f"A{i}"] = x
        cells[f"B{i}"] = f"=BETA.DIST(1-A{i},{b0!r},{a!r},FALSE)"
        cells[f"C{i}"] = f"=BETA.DIST(A{i},{a!r},{b0!r},TRUE)"
        cells[f"D{i}"] = f"=1-A{i}"
        meta.append((f"S{j}", i, x, a, b, b0))
    sheets.append({"name": f"S{j}", "cells": cells})
json.dump({"format": "rustsheet-case/v1", "description": "T-2464 w4-fractions probe 1: TOMS L140 pieces (bup density at 1-x, bpser cdf) on w2 direct runs",
           "tags": ["M9", "probe"], "sheets": sheets, "expected": {}}, open(HERE / 'w4p1.case.json', 'w'))
json.dump(meta, open(HERE / 'w4p1.meta.json', 'w'))
print(len(meta), 'rows', 3 * len(meta), 'formulas')

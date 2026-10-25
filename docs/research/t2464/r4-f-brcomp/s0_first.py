"""First look: plain brcomp/x (variants) against CR on the in-sample sets."""
import sys

import fblib as B

SETS = sys.argv[1:] or ["a", "sw", "big", "cells"]
VARIANTS = {
    "toms": {},
    "toms-xlalnrel": {"alnrel": "xl"},
    "noswap": {"swap": "none"},
    "always": {"swap": "always"},
    "div-args": {"args": "div"},
}
for s in SETS:
    rows = B.load(s)
    print(f"# {s}: {len(rows)} rows, fetched {B.prefetch_gl(rows, VARIANTS.values())} GAMMALN")
    print(B.line("CR", B.score(rows, lambda r: r["cr"])))
    for k, o in VARIANTS.items():
        print(B.line(k, B.score(rows, lambda r, o=o: B.model(r["x"], r["d1"], r["d2"], o))))

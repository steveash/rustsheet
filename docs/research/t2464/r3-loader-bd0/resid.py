"""Per-cell residual along a run: admissible C-offset intervals (units 2^-53) minus the block's best C.
Usage: resid.py [--src S] [--chain K] BLOCK 'knobs' [N]"""
import sys

import r3lib as L
import scan as S

args = sys.argv[1:]
src, chain = "old", "K"
while args[0].startswith("--"):
    if args[0] == "--src":
        src = args[1]
    else:
        chain = args[1]
    args = args[2:]
rows = S.load(src)
idx = sorted(L.blocks(rows)[args[0]], key=lambda i: rows[i]["x"])
o = S.parse(args[1] if len(args) > 1 else "")
res = L.fit_block(rows, idx, o, chain)
c = res["off_all"]
N = int(args[2]) if len(args) > 2 else 80
print(f"best C {c:+.3f} hits {res['all']}/{res['n']}")
for i in idx[:N]:
    ivs = res["ivs"][i]
    mids = " ".join(f"[{lo - c:+6.2f},{hi - c:+6.2f}]" for lo, hi in ivs)
    hit = any(lo <= c <= hi for lo, hi in ivs)
    print(f"{rows[i]['x']!r:22s} {'*' if hit else ' '} {mids}")

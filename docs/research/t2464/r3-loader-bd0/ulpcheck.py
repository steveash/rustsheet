"""Parser-misread check: for each odd-half miss of r3 (Ce) in the ll blocks, does the model match Excel at x +- 1 ulp
(same fitted constant)?  Usage: ulpcheck.py SRC"""
import math
import sys

import r3lib as L
import scan as S

o = {"v": "x", "ph": 1, "ld": "x"}
rows = S.load(sys.argv[1])
miss = hit_nb = 0
for blk, idx in sorted(L.blocks(rows).items()):
    f = L.fit_block(rows, idx, o, "Ce", W=1024.0)
    if "ll" not in f["br"]:
        continue
    L5, h = L.shape_pieces(rows[idx[0]]["a"], rows[idx[0]]["b"])
    C = L.LDB(f["C0"]) + L.LDB(f["off"]) * L.U53
    for i in idx:
        r = rows[i]
        if not r["hold"]:
            continue
        b1, b2, _ = f["X"][i]
        if L.chain_out("Ce", C, b1, b2, L5, h, None) == r["excel"]:
            continue
        miss += 1
        for x2 in (math.nextafter(r["x"], 0), math.nextafter(r["x"], 1)):
            c1, c2, _ = L.xpart(x2, r["a"], r["b"], o)
            if L.chain_out("Ce", C, c1, c2, L5, h, None) == r["excel"]:
                hit_nb += 1
                break
print(f"{sys.argv[1]}: ll odd-half misses {miss}, matched at x +- 1 ulp {hit_nb}")

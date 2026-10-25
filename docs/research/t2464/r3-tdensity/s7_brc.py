"""In-sample a1/a2: T density as Excel's own beta density machinery: brcomp(n/2, 1/2, y, z) / |t| with brcomp =
fl(P * fl(y z)) and P the M1 density at y (complement z), several orders; also P * |t| * y^2 / n.  Per df counts.
usage: s7_brc.py"""
from collections import defaultdict

import tdlib as TD

L, st = TD.LDB, TD.st
rows = [r for r in TD.load_probe("a1") + TD.load_probe("a2") if not r["blk"].startswith("df")]
res = defaultdict(lambda: defaultdict(list))
for r in rows:
    p = TD.pieces(r["t"], r["n"])
    at = L(abs(r["t"]))
    for sw in ("toms", "m1"):
        for Eo in ("e", "x"):
            o = dict(sw=sw, E=Eo)
            bc = TD.m1_brcomp(p, o)
            P = TD.m1_brcomp(p, dict(o, pdfonly=True))
            res[f"{sw},{Eo}: brc/t"][r["n"]].append((st(bc / at), r["excel"]))
            res[f"{sw},{Eo}: P*(t*y*y/n)e"][r["n"]].append((st(P * at * p["y"] * p["y"] / p["n"]), r["excel"]))
            res[f"{sw},{Eo}: P*y*z/t e"][r["n"]].append((st(P * p["y"] * p["z"] / at), r["excel"]))
            res[f"{sw},{Eo}: P*fl(y*fl(z/t))"][r["n"]].append((st(P * L(st(p["y"] * L(st(p["z"] / at))))),
                                                               r["excel"]))
for k, d in res.items():
    tot = [0, 0]
    s = []
    for n in sorted(d):
        tl = TD.tally(d[n])
        tot[0] += tl[1]
        tot[1] += tl[0]
        s.append(f"n{n:g} {tl[1]}/{tl[2]}/{tl[3]}")
    print(f"{k:28s} {tot[0]}/{tot[1]} | " + "; ".join(s))

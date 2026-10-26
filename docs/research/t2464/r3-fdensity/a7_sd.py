"""a7: per-block sd of (model - Excel) in units for R df variants (exact constant; the sd ignores the constant)
against CR, on runs of consecutive doubles.  A model that has Excel's x-dependent roundings shrinks the sd.
Usage: a7_sd.py SRC [variant ...]   SRC: tdens | probe:<name>"""
import collections
import statistics as st
import sys

import a2_struct as A2
import flib as F


def main():
    src = sys.argv[1]
    names = sys.argv[2:] or ["R", "R-bd0r2", "div", "w5"]
    if src == "tdens":
        rows = F.tdens_rows()
    else:
        import mkprobe

        rows = mkprobe.rows(src.split(":", 1)[1])
    rows = F.with_exact(rows)
    g = collections.defaultdict(list)
    for r in rows:
        g[(r["d1"], r["d2"], r["blk"])].append(r)
    print("block n | CR sd | " + " | ".join(names))
    tot = collections.defaultdict(list)
    for key, rs in sorted(g.items()):
        if len(rs) < 32:
            continue
        cr = st.pstdev([F.units(r["cr"], r["excel"]) for r in rs])
        tot["CR"].append(cr)
        out = []
        for nm in names:
            v = st.pstdev([F.units(F.df_value(r["x"], r["d1"], r["d2"], A2.VARIANTS[nm]), r["excel"]) for r in rs])
            tot[nm].append(v)
            out.append(f"{v:6.2f}")
        print(f"{key[0]:g},{key[1]:g} {key[2][:22]:22s} {len(rs):3d} | {cr:6.2f} | " + " | ".join(out))
    print("median | " + " | ".join(f"{k} {st.median(v):.2f}" for k, v in tot.items()))


if __name__ == "__main__":
    main()

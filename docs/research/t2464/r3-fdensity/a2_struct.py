"""a2: free-constant structure test of R's df (Loader dbinom_raw) for Excel's F density, per (d1, d2) block.
For each block and knob set, the largest number of cells sharing one constant C (deepest overlap of the per-cell
admissible-offset intervals; offsets in units of 2^-53 from the exact Loader constant), then C fitted on the even
md5 half and scored on the odd half.
Usage: a2_struct.py SRC [variant ...]   SRC: tdens (r3-tdensity d1 = 1 runs) | cells | probe:<name> | control"""
import collections
import hashlib
import sys

import flib as F

B3 = {"v": "x", "ph": 1, "ld": "x"}  # r3-loader-bd0's frozen bd0 knobs
VARIANTS = {
    "R": dict(pq="R", bd0=B3),
    "R-bd0r2": dict(pq="R", bd0={}),
    "R-bst": dict(pq="R", bd0=dict(B3, bst=1)),
    "div": dict(pq="div", bd0=B3),
    "w5": dict(pq="w5", bd0=B3),
    "q1": dict(pq="q1", bd0=B3),
    "p1": dict(pq="p1", bd0=B3),
    "R-pre-e": dict(pq="R", pre="e", bd0=B3),
    "R-dens-e": dict(pq="R", dens="e", bd0=B3),
    "R-E-x": dict(pq="R", E="x", bd0=B3),
    "R-np-e": dict(pq="R", np="e", bd0=B3),
}


def odd(r):
    return hashlib.md5(f"{r['blk']}|{r['x']!r}|{r['d1']}|{r['d2']}".encode()).digest()[0] & 1


def load(src):
    if src == "tdens":
        rows = F.tdens_rows()
    elif src == "cells":
        rows = F.cells_rows()
    elif src.startswith("probe:"):
        import mkprobe

        rows = mkprobe.rows(src.split(":", 1)[1])
    else:
        raise SystemExit(src)
    g = collections.defaultdict(list)
    for r in rows:
        if r["d1"] == 2:  # x = 0 branch of dbinom_raw: no Loader constant (a3_d2.py)
            continue
        key = (r["d1"], r["d2"], r["blk"] if src != "cells" else "")
        g[key].append(r)
    return {k: v for k, v in g.items() if len(v) >= 8}


def main():
    src = sys.argv[1]
    names = sys.argv[2:] or list(VARIANTS)
    blocks = load(src)
    for nm in names:
        o = VARIANTS[nm]
        tot = [0, 0, 0, 0]
        per = []
        for key, rows in sorted(blocks.items()):
            nb, pos, ivs, _ = F.fit_df_block(rows, o, W=96.0)
            ev = [iv for r, iv in zip(rows, ivs) if not odd(r)]
            od = [iv for r, iv in zip(rows, ivs) if odd(r)]
            ne, ce = F.R3.best_overlap(ev)
            nh = F.R3.hits_at(od, ce)
            tot = [tot[0] + nb, tot[1] + len(rows), tot[2] + nh, tot[3] + len(od)]
            per.append(f"{key[0]:g},{key[1]:g},{key[2][:14]}: {nb}/{len(rows)} off {pos:+.2f} hold {nh}/{len(od)}")
        print(f"{src} {nm:10s} consistent {tot[0]}/{tot[1]}  odd-half at even-fit C {tot[2]}/{tot[3]}")
        if "-v" in sys.argv[0:1] or len(names) == 1:
            print("   " + "\n   ".join(per))
        sys.stdout.flush()


if __name__ == "__main__":
    main()

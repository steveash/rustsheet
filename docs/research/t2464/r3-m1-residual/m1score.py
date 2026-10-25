"""Score the base (M1 in r2-bratio + Excel log1p) and the exponent variants per block / region.
Usage: m1score.py old|probe1|a|... [modes...]"""
import collections
import sys

import m1lib as M
from lscan import rows_of


def region(r):
    a, b = r["a"], r["b"]
    g = "b=1" if b == 1.0 else ("a=1" if a == 1.0 else ("a<1" if a < 1 else "b<1"))
    return g + (",max>8" if max(a, b) > 8 else "")


def main(name, modes):
    rows = rows_of(name)
    M.ensure_gl(rows)
    by = collections.defaultdict(lambda: collections.defaultdict(list))
    for r in rows:
        tx, ty = M.pieces(r["x"], r["a"], r["b"])
        L = M.lnb_base(r["a"], r["b"])
        g = r.get("blk", region(r))
        for m in modes:
            v = M.combine(tx, ty, L, r["a"], r["b"], m)
            by[g][m].append((v, r["excel"]))
            by["ALL"][m].append((v, r["excel"]))
    for g in sorted(by):
        print(f"{name:7s} {g:14s} " + "  ".join(f"{m}: {M.fmt(M.score(*zip(*by[g][m])))}" for m in modes))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2:] or ["m1", "dd"])

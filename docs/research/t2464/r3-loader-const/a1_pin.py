"""a1: pin a free shape constant C per (a, b) under E = fl53(C + X) (X = r2's x87 log-form bd0 part, extended).

Per cell, Excel's value gives the set of E doubles with EXP_V6(E) = excel, hence an interval for C. Per shape the
intervals are intersected greedily (largest consistent subset by a sweep over interval endpoints). Prints, per
shape: cells, consistent cells, the C interval relative to the exact constant Cex (units of 2^-53, absolute),
and LDe's constant offset for comparison.
Usage: a1_pin.py sweep|diverse [--ext] [--control DELTA]
  --ext: E extended (unstored) into EXP, C in E-interval - X; default E = fl53(C + X).
  --control: synthetic Excel values from the same structure with C = Cex + DELTA units.
Writes pin_<which>[_ext][_ctlD].tsv (one line per shape) for later scripts.
"""

from __future__ import annotations

import sys
from collections import defaultdict

import mpmath

import r3lib as R

U = mpmath.mpf(2) ** -53


def best_overlap(ivs):
    """Largest number of intervals sharing a point, and that point's interval [lo, hi]."""
    ev = sorted([(lo, 0) for lo, _ in ivs] + [(hi, 1) for _, hi in ivs])
    cur = best = 0
    blo = bhi = None
    for i, (v, t) in enumerate(ev):
        if t == 0:
            cur += 1
            if cur > best:
                best, blo = cur, v
                bhi = ev[i + 1][0] if i + 1 < len(ev) else v
        else:
            cur -= 1
    return best, blo, bhi


def shape_key(r):
    return (r["a"], r["b"])


def main():
    which = sys.argv[1]
    ctl = None
    if "--control" in sys.argv:
        ctl = float(sys.argv[sys.argv.index("--control") + 1])
    ext = "--ext" in sys.argv
    rows = R.load(which)
    by = defaultdict(list)
    for r in rows:
        by[shape_key(r)].append(r)
    out = []
    tot = cons = 0
    for (a, b), rs in sorted(by.items()):
        cex = R.exact_const(a, b)
        ivs = []
        for r in rs:
            X = R.xpart(r["x"], a, b)
            y = r["excel"]
            if ctl is not None:
                cc = cex + ctl * U + X
                y = R.L.exp_v6(cc if ext else mpmath.mpf(R.L.D(cc)))
            if ext:
                ei = R.e_interval_ext(y)
                iv = None if ei is None else (ei[0] - X, ei[1] - X)
            else:
                iv = R.c_interval(y, X)
            if iv is not None:
                ivs.append(((iv[0] - cex) / U, (iv[1] - cex) / U))
        n, lo, hi = best_overlap(ivs) if ivs else (0, None, None)
        tot += len(rs)
        cons += n
        lo_s = f"{float(lo):+9.3f}" if lo is not None else "nan"
        hi_s = f"{float(hi):+9.3f}" if hi is not None else "nan"
        out.append(f"{a!r}\t{b!r}\t{len(rs)}\t{len(ivs)}\t{n}\t{lo_s}\t{hi_s}")
    R.save_cache()
    tag = which + ("_ext" if ext else "") + (f"_ctl{ctl:g}" if ctl is not None else "")
    with open(R.HERE / f"pin_{tag}.tsv", "w") as f:
        f.write("a\tb\tcells\tinvertible\tconsistent\tlo_units\thi_units\n")
        f.write("\n".join(out) + "\n")
    print(f"{tag}: {cons}/{tot} cells consistent with one C per shape")


if __name__ == "__main__":
    main()

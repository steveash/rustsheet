"""b4: per (shape, side) needed constant from the x1 runs (mean of interval centres; overlap interval too), with
M-sd's pieces: e-part = e(h) - e(a) - e(b) + C0, side_err = M-sd's bracket. Writes need_SET.tsv.
Usage: b4_table.py SET"""

from __future__ import annotations

import sys

import numpy as np

import blib as B
import b3_reg as R

C, SD = B.C, B.SD


def table(name):
    g = R.cells(B.load(name), "x1")
    out = []
    for (m, n, side, _), cs in sorted(g.items()):
        ys = [y for y, _, _ in cs]
        full = SD.delta(m, n, side)
        epart = SD.e_of((m + n) / 2) - SD.e_of(m / 2) - SD.e_of(n / 2) + SD.C0
        out.append((m, n, side, float(np.mean(ys)), float(np.std(ys)), len(ys), full, epart, full - epart))
    return out


if __name__ == "__main__":
    t = table(sys.argv[1])
    with open(f"need_{sys.argv[1]}.tsv", "w") as f:
        f.write("d1\td2\tside\tneed\tsd\tn\tmsd_delta\tmsd_epart\tmsd_side\n")
        for row in t:
            f.write("\t".join(f"{v:g}" if isinstance(v, float) and v == int(v) and abs(v) < 1e6 and row.index(v) < 3 else str(round(v, 4)) for v in row) + "\n")
    print(len(t), "rows")

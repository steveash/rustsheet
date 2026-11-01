"""Per (b0, center, a0) on t3570-r5-bpser-lngamma-beta: does GL53's L moved by k ulps (k in -2..2) fit both x?

If the a0-linked misses (residual.out: both x miss more often than chance at 0.5 and 0.7) come from L, some
k != 0 makes both cells of that a0 exact. Prints per (b0, center): the a0 exact at k = 0, the a0 that only some k != 0
fits (with the k), and the a0 that no k in -2..2 fits.
usage: lshift.py
"""

from __future__ import annotations

import json
import math
from collections import Counter, defaultdict

import make_probe as mp
import score_forms as score

lib = mp.lib
SHIFT = [0]
_base = lib.Model.lnb_w3


def shifted(self, a, b):
    lv = _base(self, a, b)
    f = float(lv)
    return lib.mpf(f + SHIFT[0] * math.ulp(f))


def main() -> None:
    lib.bx._GL.update(score.measured_gl())
    lib.Model.lnb_w3 = shifted
    out = score.sidecar("t3570-r5-bpser-lngamma-beta")
    cells = defaultdict(list)
    for i, blk, x, a, b in json.load(open(mp.HERE / "t3570-r5-bpser-lngamma-beta.meta.json")):
        b0, c, _ = blk.split(":")
        cells[(b0, c, a)].append((x, b, float(out[f"A{i}"]["v"])))
    fits = defaultdict(set)
    for k in (-2, -1, 0, 1, 2):
        SHIFT[0] = k
        lib._MODELS.clear()
        v = lib.V(lg=f"GL53#{k}")  # distinct key per shift
        for key, cs in cells.items():
            if all(lib.kernel(key[2], b, x, 1.0 - x, v)[0] == e for x, b, e in cs):
                fits[key].add(k)
    summary = defaultdict(Counter)
    for key in cells:
        f = fits[key]
        tag = "k=0" if 0 in f else ("shift " + ",".join(map(str, sorted(f))) if f else "none")
        summary[key[:2]][tag] += 1
    for key in sorted(summary):
        print(":".join(key), dict(sorted(summary[key].items())))


if __name__ == "__main__":
    main()

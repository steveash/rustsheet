"""verify-r3-loader-const: bit-exact / w4 / w15 of each frozen model per |ln y| bin and data group.
Tests the "extended E" claim with no free parameter: at |E| >= 8 a stored E cannot be bit-exact anywhere (vimage.py),
at smaller |E| stored and extended E differ only through EXP V6's sensitivity.
Usage: vbins.py [MODEL ...]
"""

from __future__ import annotations

import math
import sys
from collections import defaultdict

import vdata
import vscore


def ebin(y: float) -> str:
    L = abs(math.log(y))
    return "<1" if L < 1 else "1-2" if L < 2 else "2-4" if L < 4 else "4-8" if L < 8 else ">=8"


def main():
    names = sys.argv[1:] or ["M4", "M4-stE", "M4-Cd", "M4-Xd", "M4-CdXd", "M3", "Mexact", "CR"]
    rows, _ = vdata.load_all()
    rows, _, _ = vscore.dedup(rows)
    P = vscore.predict(rows)
    by = defaultdict(list)
    for r in rows:
        g = vscore.group(r["case"])
        g = "worker-insample" if g.startswith("worker-insample") else g
        if g in ("r2-era", "other-r3-probes"):
            g = "not-seen-by-worker"
        by[(g, ebin(r["excel"]))].append(r)
        by[(g, "all")].append(r)
    for g in ("worker-insample", "worker-hold", "verifier-hold", "not-seen-by-worker"):
        for eb in ("all", "<1", "1-2", "2-4", "4-8", ">=8"):
            rs = by.get((g, eb))
            if not rs:
                continue
            print(f"== {g} |E| {eb} (n {len(rs)})")
            for nm in names:
                t = vscore.tally((P[(r["x"], r["a"], r["b"])][nm], r["excel"]) for r in rs)
                print(f"  {nm:8s} {vscore.fmt(t)}")


if __name__ == "__main__":
    main()

"""Verifier side checks: (1) Excel vs CR at x = 1 exactly (ulp histogram); (2) C_BIG sweep on each set's x1 cells,
to see whether the frozen -0.42 is near the out-of-sample optimum; (3) the e-table part M-sd adds at these shapes.
Usage: vchecks.py CASE [CASE ...]"""

from __future__ import annotations

import collections
import sys

import vscore as S
from vparse import rows

if __name__ == "__main__":
    bc = S.importlib.import_module("bcmodel") if S.sys.path.insert(0, str(S.WORKER)) is None else None
    for case in sys.argv[1:]:
        rs = [r for r in rows(case) if r["excel"] > 0 and r["echo_ok"]]
        one = [r for r in rs if r["x"] == 1.0]
        h = collections.Counter()
        for r in one:
            cr = float(S.exact(1.0, r["d1"], r["d2"]))
            h[S.ordkey(r["excel"]) - S.ordkey(cr)] += 1
        print(f"== {case}: x=1 cells {len(one)}; Excel - CR (ulps): {sorted(h.items())}")
        x1 = [r for r in rs if S.cls(r) == "x1"]
        line = []
        for c in (-1.5, -1.0, -0.7, -0.42, -0.2, 0.0, 0.3, 0.6, 1.0, 1.54):
            bc.C_BIG = c
            bc.K_of.cache_clear()
            line.append(f"{c:+.2f}:{sum(bc.value(r['x'], r['d1'], r['d2'], 'div') == r['excel'] for r in x1)}")
        print(f"  C_BIG sweep, x1 bit-exact of {len(x1)}: " + " ".join(line))
        bc.C_BIG = -0.42
        bc.K_of.cache_clear()
        et = sorted({round(bc.SD.delta(r["d1"], r["d2"], s) - bc.side_err(r["d1"], r["d2"], s), 3)
                     for r in x1 for s in (1, 2)})
        print(f"  M-sd e-table part at these shapes (units): min {et[0]} max {et[-1]} distinct {len(et)}")
    S.save()

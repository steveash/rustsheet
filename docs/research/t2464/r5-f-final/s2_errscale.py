"""T-2586 r5-f-final diagnostic: M-final's error (model vs Excel, units of 2^-53 relative) by |P| bin on all earlier
sets and the final hold-out, against the spread a single extra rounding at P's scale would give
(ulp(|P|)/sqrt(12) in units, since an absolute error in E is a relative error in F).
Usage: s2_errscale.py"""

from __future__ import annotations

import math

import numpy as np

import final_score as FS
import s1_offmap as S1

M, F = FS.M, FS.F


def main():
    sets = FS.set_rows()
    sets["final-hold"] = FS.hold_rows()[0]
    acc: dict = {}
    for rows in sets.values():
        for r in rows:
            m, n, x = math.trunc(r["d1"]), math.trunc(r["d2"]), r["x"]
            if isinstance(r["excel"], str) or r["excel"] < 1e-290 or M.rule(x, m, n) is not None:
                continue
            side, _ = M.SL.side_r(x, float(m), float(n))
            P = abs((m / 2 if side == 1 else n / 2) * math.log(x))
            v = M.value(x, m, n)
            if not v or isinstance(v, str):
                continue
            acc.setdefault(S1.pbin(P), []).append((F.units(v, r["excel"]), P))
    print("|P| bin           n    rms units   median |err|   mean ulp(P)/sqrt12 (units)")
    for k in sorted(acc, key=lambda s: float(s.strip("<[").split(",")[0])):
        e = np.array([u for u, _ in acc[k]])
        q = np.mean([math.ulp(P) * 2**53 / math.sqrt(12) for _, P in acc[k]])
        print(f"{k:14s} {len(e):6d} {np.sqrt(np.mean(e**2)):10.2f} {np.median(np.abs(e)):12.2f} {q:14.2f}")


if __name__ == "__main__":
    main()

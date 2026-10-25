"""verify-r4-f-small-df: is M-sd's delta the constant Excel uses? Per (shape, side) group of a case, scan a free
offset (units of 2^-53 on K, step 0.25 over -12..+16) in my recomposition vM-sd and report the bit-exact-maximising
interval against the model's delta. Positive control (--plant): replace Excel's values by vM-sd with delta + 2.5
and check the scan recovers the planted shift.
Usage: vconst.py CASE_ID [--plant]"""

from __future__ import annotations

import sys

import numpy as np

import vscore as V

LDB = np.longdouble
GRID = [k / 4 for k in range(-48, 65)]


def val(x, m, n, side, dK):
    """vM-sd with K shifted by dK units."""
    mx = V.WL.st(LDB(m) * LDB(x))
    r = V.WL.st(LDB(mx) / LDB(n)) if side == 1 else V.WL.st(LDB(n) / LDB(mx))
    c = m / 2 if side == 1 else -n / 2
    K = V.vm_K(m, n, side) + LDB(dK) * LDB(2.0) ** -53
    E = K + LDB(V.WL.st(LDB(c) * V.WL.LN(x))) - LDB((m + n) / 2) * V.WL.L_xl(r)
    return float(V.WL.st(LDB(V.WL.EXP(E)) / LDB(x)))


def main(cid: str, plant: bool) -> None:
    groups: dict = {}
    for r in V.rows(cid):
        side = 1 if V.WL.st(LDB(r["d1"]) * LDB(r["x"])) <= r["d2"] else 2
        if plant:
            r["excel"] = val(r["x"], r["d1"], r["d2"], side, 2.5)
        groups.setdefault((r["d1"], r["d2"], side), []).append(r)
    offs = []
    for (m, n, s), rs in sorted(groups.items()):
        sc = [sum(val(r["x"], m, n, s, g) == r["excel"] for r in rs) for g in GRID]
        best = max(sc)
        arg = [g for g, c in zip(GRID, sc) if c >= best - 1]
        mid = (min(arg) + max(arg)) / 2
        offs.append(mid)
        print(
            f"({m:g},{n:g}) side {s}: n {len(rs)} at model delta {sc[GRID.index(0.0)]}, best {best} at "
            f"{min(arg):+.2f}..{max(arg):+.2f} (mid {mid:+.2f})"
        )
    a = np.array(offs)
    print(
        f"groups {len(a)}: free-offset mid relative to M-sd's delta: mean {a.mean():+.2f}, RMS {np.sqrt((a**2).mean()):.2f}, "
        f"within 1 unit {int((abs(a) <= 1).sum())}"
    )


if __name__ == "__main__":
    main(sys.argv[1], "--plant" in sys.argv)

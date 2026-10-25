"""b2: per-cell needed delta (units, against exact K) under M-sd's structure, for chosen runs.
Usage: b2_cells.py SET d1 d2 [run]   (run 'x1' or 'sw')"""

from __future__ import annotations

import sys

import blib as B

C, SL = B.C, B.SL

if __name__ == "__main__":
    rows = B.load(sys.argv[1])
    m, n = float(sys.argv[2]), float(sys.argv[3])
    run = sys.argv[4] if len(sys.argv) > 4 else None
    rs = [r for r in rows if r["d1"] == m and r["d2"] == n and (run is None or r["blk"].startswith(run))]
    for pc, E0, (lo, hi), r in C.run_cells(rs):
        K = C.K_exact(m, n, pc["side"])
        base = float((E0 - C.ex(pc["P"]) - C.ex(pc["H"]) - K) * B.U)
        s = SL.st(SL.LDB(1) + SL.LDB(pc["r"]))
        print(f"{r['blk']:12s} x={r['x']!r:22s} side={pc['side']} r={pc['r']!r:22s} s={s!r:20s} P={float(pc['P']):+.3e} "
              f"need=[{base + lo:8.2f},{base + hi:8.2f}] w={hi - lo:.2f}")

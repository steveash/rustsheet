"""b1: the constant Excel needs per (shape, side, run) under M-sd's x-structure (E = K + P + H, H = -h L(r) extended,
F = fl(EXP(E)/x)), by interval inversion; compared with M-sd's delta and the exact K.
Usage: b1_need.py SET [--plant DELTA]   (--plant: replace Excel by M-sd with delta + DELTA units, positive control)"""

from __future__ import annotations

import sys

import blib as B

C, SL, SD = B.C, B.SL, B.SD


def plant_value(r, plant: float) -> float:
    """M-sd's chain with delta + plant units, through clib's final step (EXP extended, then /x)."""
    pc = C.pieces(r["x"], r["d1"], r["d2"])
    K = C.K_sd(r["d1"], r["d2"], pc["side"]) + B.mpf(plant) / B.U
    return C.final(SL.ld(K + C.ex(pc["P"]) + C.ex(pc["H"])), r["x"])


def need_rows(rows, plant=None):
    if plant is not None:
        rows = [dict(r, excel=plant_value(r, plant)) for r in rows]
    g: dict = {}
    for pc, E0, (lo, hi), r in C.run_cells(rows):
        K = C.K_exact(r["d1"], r["d2"], pc["side"])
        base = float((E0 - C.ex(pc["P"]) - C.ex(pc["H"]) - K) * B.U)
        g.setdefault((r["d1"], r["d2"], pc["side"], r["blk"][:2]), []).append((base + lo, base + hi, pc["r"]))
    C.save()
    out = []
    for k, ivs in sorted(g.items()):
        n, lo, hi = C.best_overlap([(a, b) for a, b, _ in ivs])
        out.append((k, len(ivs), n, lo, hi, min(r for *_, r in ivs), SD.delta(k[0], k[1], k[2])))
    return out


if __name__ == "__main__":
    plant = float(sys.argv[sys.argv.index("--plant") + 1]) if "--plant" in sys.argv else None
    res = need_rows(B.load(sys.argv[1]), plant)
    print("d1 d2 side run  cells fit  lo hi  r  Msd_delta  need-Msd")
    for (m, n, s, run), N, k, lo, hi, r, d in res:
        mid = (lo + hi) / 2 if lo is not None else float("nan")
        print(f"{m:5g} {n:5g} {s} {run} {N:3d} {k:3d} {lo:9.2f} {hi:9.2f} r={r:.3f} sd={d:8.2f} diff={mid - d:7.2f}")

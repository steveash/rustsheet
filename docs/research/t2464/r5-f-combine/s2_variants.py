"""s2: chain variants on r > 0.375 runs, free constant per run (and free K_hi phase where the variant stores a sum
with the constant).  Each variant maps (K_hi, pieces) -> S (the part of E that is not the remainder K_lo); E = S +
K_lo, K_lo free (interval overlap).  Printed: per variant, total best-overlap count over the runs, and runs fully fit.
Variants (H = -h L, hs = -fl(h L), P stored product as entering):
  ext     P + H                         (M-sd's structure, extended)
  hkp     fl(fl(Khi + H) + P)           (r4-f-power's S2)
  hkp_hs  fl(fl(Khi + hs) + P)
  kph     fl(fl(Khi + P) + H)
  ph      fl(P + H)                     (constant outside)
  hk      fl(Khi + H) + P               (P added in extended)
  hs      P + hs                        (h L product stored)
  hsp     fl(P + hs)
  hk_hs   fl(Khi + hs) + P
Usage: s2_variants.py SET VARIANT... [--by-run]"""

import sys

import numpy as np

import clib as C
import s1_scan as S1

LDB, st = C.LDB, C.st
NEEDS_K = {"hkp", "hkp_hs", "kph", "hk", "hk_hs"}


def S_of(var, Khi, pc):
    H, P = pc["H"], LDB(pc["P"])
    hs = LDB(-st(-H))
    K = LDB(Khi)
    if var == "ext":
        return P + H
    if var == "hkp":
        return LDB(st(LDB(st(K + H)) + P))
    if var == "hkp_hs":
        return LDB(st(LDB(st(K + hs)) + P))
    if var == "kph":
        return LDB(st(LDB(st(K + P)) + H))
    if var == "ph":
        return LDB(st(P + H))
    if var == "hk":
        return LDB(st(K + H)) + P
    if var == "hs":
        return P + hs
    if var == "hsp":
        return LDB(st(P + hs))
    if var == "hk_hs":
        return LDB(st(K + hs)) + P
    raise ValueError(var)


def fit(cells, m, n, side, var):
    Kref = C.K_exact(m, n, side)
    kh0 = float(Kref)
    if var not in NEEDS_K:
        return _count(cells, var, 0.0 if var in ("ext", "hs") else kh0, Kref, var)
    g = [S1.ulp_units(st(LDB(kh0) + pc["H"])) for pc, *_ in cells]
    G = max(max(g), max(S1.ulp_units(pc["P"]) for pc, *_ in cells), 1.0)
    step = max(S1.ulp_units(kh0), min(g) / 32)
    best = (0,)
    seen = set()
    for t in np.arange(-G, G + step / 2, step):
        Khi = float(Kref + C.mpf(t) / C.U)
        if Khi in seen:
            continue
        seen.add(Khi)
        r = _count(cells, var, Khi, Kref, var)
        if r[0] > best[0]:
            best = r + (t,)
    return best


def _count(cells, var, Khi, Kref, _v):
    kh = C.ex(Khi) if var in NEEDS_K else 0
    ivs = []
    for pc, E0, (lo, hi), _ in cells:
        k = float((E0 - C.ex(S_of(var, Khi, pc)) + kh - Kref) * C.U)
        ivs.append((k + lo, k + hi))
    return C.best_overlap(ivs)


def main():
    a = sys.argv[1:]
    name, vars_ = a[0], [v for v in a[1:] if not v.startswith("--") and v not in C.FINS]
    fin = next((v for v in a[1:] if v in C.FINS), "div")
    tot = {v: [0, 0, 0] for v in vars_}
    for key, rs in sorted(C.runs(C.load(name)).items()):
        m, n, side, _ = key
        cells = C.run_cells(rs, fin)
        if len(cells) < 8:
            continue
        line = []
        for v in vars_:
            b = fit(cells, m, n, side, v)
            tot[v][0] += b[0]
            tot[v][1] += len(cells)
            tot[v][2] += b[0] == len(cells)
            line.append(f"{v} {b[0]:2d} d{(b[1] or 0):+7.1f}")
        if "--by-run" in a:
            print(f"({m:g},{n:g}) s{side} x{rs[0]['x']:.4g} n{len(cells)} | " + " | ".join(line))
            sys.stdout.flush()
    for v, (c, nn, full) in tot.items():
        print(f"{name} {fin} {v:7s} {c}/{nn} cells, {full} runs full")
    C.save()


if __name__ == "__main__":
    main()

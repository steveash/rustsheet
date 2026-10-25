"""s5: flexible stored-sum search on r > 0.375 runs, scored by interval overlap (free remainder K_lo per run).
Terms: K (a double: fl(K_exact) moved j ulps, j scanned over +-J), P = sg*fl(c LN x), and an H form:
  He  -h*L extended            Hs  -fl(h*L)
  Hab -(fl(a*L) + fl(b*L)) (sum extended)       Habs -fl(fl(a*L) + fl(b*L))
  Hl  -h*L with L = LN(s) kept extended (LN64)  (control)
The order lists the summands; flags give per partial sum stored (1) / extended (0).  E = sum + K_lo.
Printed: per (H form, order, flags) total count over the runs, best few.  Usage: s5_flex.py SET J [MAXD] [--side S]"""

import itertools
import sys

import numpy as np

import clib as C

LDB, st = C.LDB, C.st
HFORMS = ("He", "Hs", "Hab", "Habs")


def H_of(pc, form):
    L = C.SL.L_xl(pc["r"])
    a, b, h = LDB(pc["a"]), LDB(pc["b"]), LDB(pc["h"])
    if form == "He":
        return -h * L
    if form == "Hs":
        return -LDB(st(h * L))
    if form == "Hab":
        return -(LDB(st(a * L)) + LDB(st(b * L)))
    if form == "Habs":
        return -LDB(st(LDB(st(a * L)) + LDB(st(b * L))))
    raise ValueError(form)


def combos():
    for order in itertools.permutations("KHP"):
        for flags in itertools.product((0, 1), repeat=2):
            yield "".join(order), flags


def total(t, order, flags):
    acc = t[order[0]]
    for i, k in enumerate(order[1:]):
        acc = acc + t[k]
        if flags[i]:
            acc = LDB(st(acc))
    return acc


def run_scores(cells, m, n, side, J):
    """{(hform, order, flags): best count over j}."""
    Kref = C.K_exact(m, n, side)
    k0 = float(Kref)
    Ks = [k0]
    for d in (1, -1):
        k = k0
        for _ in range(J):
            k = float(np.nextafter(k, d * np.inf))
            Ks.append(k)
    out = {}
    pre = [(pc, E0, iv, {f: H_of(pc, f) for f in HFORMS}) for pc, E0, iv, _ in cells]
    for hf in HFORMS:
        for order, flags in combos():
            if not any(flags) and order != "KHP":
                continue  # all-extended orders are equivalent
            best = 0
            for K in Ks:
                ivs = []
                for pc, E0, (lo, hi), Hd in pre:
                    t = {"K": LDB(K), "H": Hd[hf], "P": LDB(pc["P"])}
                    kk = float((E0 - C.ex(total(t, order, flags)) + C.ex(K) - Kref) * C.U)
                    ivs.append((kk + lo, kk + hi))
                best = max(best, C.best_overlap(ivs)[0])
                if not any(flags[max(order.index("K"), 1) - 1 :]):
                    break  # no stored sum at or after K's addition: K's phase cannot matter
            out[(hf, order, flags)] = best
    return out


def main():
    a = sys.argv[1:]
    name, J = a[0], int(a[1])
    maxd = float(a[2]) if len(a) > 2 and not a[2].startswith("--") else 1e9
    side_only = int(a[a.index("--side") + 1]) if "--side" in a else 0
    tot: dict = {}
    nn = 0
    for key, rs in sorted(C.runs(C.load(name)).items()):
        m, n, side, _ = key
        if max(m, n) > maxd or side_only and side != side_only:
            continue
        cells = C.run_cells(rs)
        if len(cells) < 8:
            continue
        sc = run_scores(cells, m, n, side, J)
        nn += len(cells)
        for k, v in sc.items():
            tot[k] = tot.get(k, 0) + v
        b = max(sc.items(), key=lambda kv: kv[1])
        print(
            f"({m:g},{n:g}) s{side} x{rs[0]['x']:.4g} n{len(cells)} best {b[0][0]} {b[0][1]}{''.join(map(str, b[0][2]))} {b[1]}"
            f" | He-ext {sc[('He', 'KHP', (0, 0))]}"
        )
        sys.stdout.flush()
    for k, v in sorted(tot.items(), key=lambda kv: -kv[1])[:20]:
        print(f"TOTAL {k[0]:5s} {k[1]}{''.join(map(str, k[2]))} {v}/{nn}")
    C.save()


if __name__ == "__main__":
    main()

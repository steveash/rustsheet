"""s4: free stored-sum magnitude.  For r > 0.375, H = -h L(s) depends on x only through s = fl(1+r), so a stored
sum T = fl(Q + H) is constant on s-plateaus.  For each binade 2^k <= |T| < 2^(k+1) (both signs) scan the phase of
a free constant Q over one ulp of T (32 steps) and fit the remainder freely:
  'hk'  E = fl(Q + H) + P + K_lo
  'hkp' E = fl(fl(Q + H) + P) + K_lo
  'pk'  E = fl(Q + P) + H + K_lo;  'phk' E = fl(Q + (P + H)) + K_lo;  'phsk' E = fl(Q + fl(P + H)) + K_lo
Printed per run: the best count per binade (the max over phase), the winner.  Usage: s4_qscan.py SET [VARIANT]
[--shape m,n]"""

import sys

import numpy as np

import clib as C

LDB, st = C.LDB, C.st


def count(cells, Q, var, Kref):
    ivs = []
    for pc, E0, (lo, hi), _ in cells:
        H, P = pc["H"], LDB(pc["P"])
        if var == "hk":
            S = LDB(st(LDB(Q) + H)) + P
        elif var == "hkp":
            S = LDB(st(LDB(st(LDB(Q) + H)) + P))
        elif var == "pk":
            S = LDB(st(LDB(Q) + P)) + H
        elif var == "phk":
            S = LDB(st(LDB(Q) + (P + H)))
        elif var == "phsk":
            S = LDB(st(LDB(Q) + LDB(st(P + H))))
        else:
            raise ValueError(var)
        k = float((E0 - C.ex(S) + C.ex(Q) - Kref) * C.U)  # relative to K_exact: keeps the float exact
        ivs.append((k + lo, k + hi))
    return C.best_overlap(ivs)[0]


def scan(cells, var, Kref, ks=range(-3, 10)):
    Hm = float(np.mean([float(pc["H"]) for pc, *_ in cells]))
    out = {}
    for k in ks:
        for sg in (1, -1):
            T0 = sg * 1.5 * 2.0**k
            u = float(np.spacing(np.float64(abs(T0))))
            best = max(count(cells, T0 - Hm + j * u / 32, var, Kref) for j in range(32))
            out[(sg, k)] = best
    return out


def main():
    a = sys.argv[1:]
    name = a[0]
    var = a[1] if len(a) > 1 and not a[1].startswith("--") else "hk"
    want = tuple(float(v) for v in a[a.index("--shape") + 1].split(",")) if "--shape" in a else None
    for key, rs in sorted(C.runs(C.load(name)).items()):
        m, n, side, _ = key
        if want and (m, n) != want:
            continue
        cells = C.run_cells(rs)
        if len(cells) < 12:
            continue
        res = scan(cells, var, C.K_exact(m, n, side))
        (bs, bk), bc = max(res.items(), key=lambda kv: kv[1])
        line = " ".join(f"{'+' if s > 0 else '-'}{k}:{c}" for (s, k), c in sorted(res.items()))
        Kref = float(C.K_exact(m, n, side))
        print(
            f"({m:g},{n:g}) s{side} x{rs[0]['x']:.4g} n{len(cells)} K{Kref:+.2f} H{float(cells[0][0]['H']):+.2f} "
            f"best {'+' if bs > 0 else '-'}2^{bk}:{bc} | {line}"
        )
        sys.stdout.flush()
    C.save()


if __name__ == "__main__":
    main()

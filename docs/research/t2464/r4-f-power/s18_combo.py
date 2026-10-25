"""s18: structure enumeration of the side-2 (mx > n) x-power: sum of up to three stored products t_i*fl(c_i*LN(x))
plus an unstored k*LN(x), with sum t_i c_i + k = -(b+1).  c_i from {1, a-1, a, a+1, b-1, b, b+1, h-1, h, h+1}.
Scored by the median over runs of the residual sd (Excel - model).  Usage: s18_combo.py SRC MINMAX TAG"""
import collections
import itertools
import sys

import mpmath
import numpy as np

import cands as C
import m1_fixed as M
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
U = P.U


SYM = {"1": (0, 0, 1), "a-1": (1, 0, -1), "a": (1, 0, 0), "a+1": (1, 0, 1), "b-1": (0, 1, -1), "b": (0, 1, 0),
       "b+1": (0, 1, 1), "h-1": (1, 1, -1), "h": (1, 1, 0), "h+1": (1, 1, 1)}


def coefs(a, b, h):
    return {k: v[0] * a + v[1] * b + v[2] for k, v in SYM.items()}


def main():
    src, minmax, tag = sys.argv[1], float(sys.argv[2]), sys.argv[3]
    runs = []
    for blk, rs, m, n in P.runs(src, minmax, tag):
        rs = [r for r in rs if m * r["x"] > n]
        if len(rs) < 20:
            continue
        a, b, h = m / 2, n / 2, (m + n) / 2
        D = np.array([P.D_units(r) for r in rs])
        base = []
        Lxs = []
        for r in rs:
            s, _ = C.pieces(r["x"], m, n)
            Kp = s3_order.kprime(m, n, False)
            base.append((Kp, -LDB(h) * M.L_form(s, False, "xl", n)))
            Lxs.append(s["Lx"])
        ex_ln = [mpmath.log(r["exact"]) for r in rs]
        runs.append((blk, a, b, h, D, base, Lxs, ex_ln))
    names = list(coefs(1, 1, 1))
    scores = collections.defaultdict(list)
    for blk, a, b, h, D, base, Lxs, ex_ln in runs:
        cs = coefs(a, b, h)
        prods = {k: [S(LDB(c) * L) for L in Lxs] for k, c in cs.items()}
        -(b + 1)
        for nt in (1, 2, 3):
            for combo in itertools.combinations(names, nt):
                for signs in itertools.product((1, -1), repeat=nt):
                    vec = [sum(sg * SYM[c][i] for sg, c in zip(signs, combo)) for i in range(3)]
                    if vec[0] != 0 or vec[1] != -1 or abs(-1 - vec[2]) > 1:  # identically -(b+1), +-1 LN(x) left
                        continue
                    k = -1 - vec[2]
                    key = " ".join(f"{'+' if sg > 0 else '-'}{c}" for sg, c in zip(signs, combo)) + f" {k:+g}L"
                    Dm = []
                    for j in range(len(D)):
                        v = sum(sg * prods[c][j] for sg, c in zip(signs, combo)) + LDB(k) * Lxs[j] + base[j][1]
                        Dm.append(float((base[j][0] + ex(v) - ex_ln[j]) * U))
                    scores[key].append(((D - np.array(Dm)).std(), blk))
    tab = sorted(scores.items(), key=lambda kv: np.median([v[0] for v in kv[1]]))
    print("runs:", [r[0] for r in runs])
    for k, v in tab[:15]:
        print(f"{k:28s} median sd {np.median([s for s, _ in v]):7.2f}  per run " + " ".join(f"{s:.0f}" for s, _ in v))


if __name__ == "__main__":
    main()

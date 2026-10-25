"""b2: is Excel's F density its Loader beta density evaluated at (y, q) with q formed separately, times a Jacobian?
Free per-block constant C in the density's exponent (E = C + X extended into EXP V6; the density stored), then the
value-space Jacobian; per block the largest number of cells sharing one C, and the odd md5 half at the even-half C.
Only min(d1, d2) >= 3 blocks (both beta shapes > 1).
Usage: b2_bj.py SRC [yqform:jac ...]   SRC: probe:a | cells"""
import itertools
import sys

import a2_struct as A2
import flib as F

LDB = F.LDB
B3 = {"v": "x", "ph": 1, "ld": "x"}


def fit(rows, yqf, jac, ob=B3, target=None, W=96.0):
    m, n = rows[0]["d1"], rows[0]["d2"]
    a, b = m / 2, n / 2
    pre = []
    for r in rows:
        y, q = F.yq(r["x"], m, n, yqf)
        X, _ = F.loader_xq(y, q, a, b, ob)
        pre.append((y, q, X))
    # reference C: exact Loader constant (beta (a, b)) at the middle cell, as the gauge
    r0 = rows[len(rows) // 2]
    y0, q0, X0 = pre[len(rows) // 2]
    C0 = F.R3._ld_of(F.R3.exact_logpdf(float(y0), a, b)) - X0
    ivs = []
    for j, (r, (y, q, X)) in enumerate(zip(rows, pre)):
        def out(u, X=X, y=y, q=q, r=r):
            d = LDB(F.st(F.R3.exp_v6(C0 + LDB(u) * F.U53 + X, True)))
            return F.jac_out(d, r["x"], m, n, y, q, jac)
        ivs.append(F.TD.interval(out, r["excel"] if target is None else target[j], W))
    nb, pos = F.R3.best_overlap(ivs)
    ev = [iv for r, iv in zip(rows, ivs) if not A2.odd(r)]
    od = [iv for r, iv in zip(rows, ivs) if A2.odd(r)]
    ne, ce = F.R3.best_overlap(ev)
    return nb, pos, F.R3.hits_at(od, ce), len(od)


def main():
    src = sys.argv[1]
    combos = [c.split(":") for c in sys.argv[2:]] or list(itertools.product(("w5", "div"), ("dyq/x", "mn/den2")))
    blocks = {k: v for k, v in A2.load(src).items() if min(k[0], k[1]) >= 3}
    for yqf, jac in combos:
        tot = [0, 0, 0, 0]
        per = []
        for key, rows in sorted(blocks.items()):
            nb, pos, nh, nod = fit(rows, yqf, jac)
            tot = [tot[0] + nb, tot[1] + len(rows), tot[2] + nh, tot[3] + nod]
            per.append(f"{key[0]:g},{key[1]:g},{key[2][-7:]} {nb}/{len(rows)} off {pos:+.1f}")
        print(f"{src} {yqf}:{jac} consistent {tot[0]}/{tot[1]}  odd-half {tot[2]}/{tot[3]}")
        print("   " + "; ".join(per))
        sys.stdout.flush()


if __name__ == "__main__":
    main()

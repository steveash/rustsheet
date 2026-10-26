"""Product forms at a = 1: density = fl(P(ty) op C) with P = Excel EXP of ty (stored double, or extended) and C a
per-shape constant double scanned over +-J ulps of exp(-lnB) (op *) or exp(lnB) (op /).
Usage: cprod.py log1p|probe1 [step] [J]"""
import collections
import sys

import mpmath

import m1lib as M

mpf, R64, R53, W1 = M.mpf, M.fl64, M.fl53, M.W1


def run(rows, J):
    by = collections.defaultdict(list)
    for r in rows:
        by[r["b"]].append(r)
    tot = collections.Counter()
    for b, rs in sorted(by.items()):
        L = mpf(M.lnb_base(1.0, b))
        tys = [mpf(M.sx("*", b - 1.0, r["l"])) if "l" in r else mpf(M.pieces(r["x"], 1.0, b)[1]) for r in rs]
        P53 = [mpf(W1.exp_v6(t)) for t in tys]
        P64 = [W1.exp_v6_ext(t) for t in tys]
        cm, cd = float(mpmath.exp(-L)), float(mpmath.exp(L))
        best = {}
        for form in ("P53*C", "P64*C", "P53/C", "P64/C"):
            c0 = cm if form.endswith("*C") else cd
            res = []
            for j in range(-J, J + 1):
                C = mpf(M.nb(c0, j))
                P = P53 if form.startswith("P53") else P64
                if form.endswith("*C"):
                    h = sum(R53(p * C) == r["excel"] for p, r in zip(P, rs))
                else:
                    h = sum(R53(p / C) == r["excel"] for p, r in zip(P, rs))
                res.append((h, j))
            best[form] = max(res)
            tot[form] += best[form][0]
        base = sum(M.combine(0.0, t, L, 1.0, b) == r["excel"] for t, r in zip(tys, rs))
        tot["m1"] += base
        print(f"b={b:.0f} n={len(rs)} m1={base} " + " ".join(f"{k}:{v[0]}@{v[1]:+d}" for k, v in best.items()))
    print("total", len(rows), dict(tot))


if __name__ == "__main__":
    name = sys.argv[1]
    step = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    J = int(sys.argv[3]) if len(sys.argv) > 3 else 6
    rows = M.load_log1p_rows() if name == "log1p" else [r for r in M.load_probe1() if r["a"] == 1.0]
    run(rows[::step], J)

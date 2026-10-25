"""Orders of Excel's exponent sum in the clean region (H ~ 0): E = (((t1 + t2) + ... + tj) + P) + t(j+1) + ... with
every addition stored. The pre-P chain rounds as one phase, each later term is rounded to the grid q on its own,
so D(q) = C - psi_q(pre) - sum psi_q(t_i). Decompositions of the constant (all their terms must appear):
  L3  A, B1, ST          A = fl((h-1/2) L(fl(w/o))), B1 = fl(0.5 LN(fl(w/2pi))), ST = fl(fl(st(h) - st(a)) - st(b))
  L4  A, Bw, -ln sqrt(2pi), ST      Bw = fl(0.5 LN w)
  L5  A, B1, st(h), -st(a), -st(b)
  L6  A, Bw, -ln sqrt(2pi), st(h), -st(a), -st(b)
  G4  GL(h), -GL(a), -GL(b), fl(c LN(fl(m/n)))
  G2  fl(fl(GL(h) - GL(a)) - GL(b)), fl(c LN(m/n))
  K1  fl(K')
Every permutation and P position is scored (groups within TOL, all 46 clean groups with >= 2 grid sizes).
--plant DEC:IDX plants that ordering (positive control). Usage: s12_orders.py [--tol 3.5] [--plant L5:123]"""

from __future__ import annotations

import itertools
import random
import sys

import numpy as np

import qbits as QB
import s11_qsearch as S11

L, LDB, st = QB.L, QB.LDB, QB.st
TOL = float(sys.argv[sys.argv.index("--tol") + 1]) if "--tol" in sys.argv else 3.5
PLANT = sys.argv[sys.argv.index("--plant") + 1] if "--plant" in sys.argv else None


def decomps(m: float, n: float, side: int) -> dict:
    p = S11.pieces(m, n, side)
    a, b, h = m / 2, n / 2, (m + n) / 2
    w = a if side == 1 else b
    B1 = st(LDB(0.5) * L.LN(st(LDB(w) / LDB(2 * float(L.mpmath.pi)))))
    ST = st(LDB(st(LDB(p["sth"]) - LDB(p["sta"]))) - LDB(p["stb"]))
    G2 = st(LDB(st(LDB(p["gh"]) - LDB(p["ga"]))) - LDB(p["gb"]))
    return {
        "L3": [p["A"], B1, ST],
        "L4": [p["A"], p["hLwo"], -p["lnsq2pi"], ST],
        "L5": [p["A"], B1, p["sth"], -p["sta"], -p["stb"]],
        "L6": [p["A"], p["hLwo"], -p["lnsq2pi"], p["sth"], -p["sta"], -p["stb"]],
        "G4": [p["gh"], -p["ga"], -p["gb"], p["cLNmn"]],
        "G2": [G2, p["cLNmn"]],
        "K1": [float(L.K_exact(m, n, side))],
    }


def orderings(terms):
    """(label, [phase values]): pre-P stored chain as one value, then each post-P term."""
    k = len(terms)
    for perm in itertools.permutations(range(k)):
        for j in range(k + 1):
            pre = 0.0
            for i in perm[:j]:
                pre = st(LDB(pre) + LDB(terms[i]))
            yield f"{perm}|P@{j}", [pre] + [terms[i] for i in perm[j:]]


def psi_sum(vals, q: float) -> float:
    s = 0.0
    for v in vals:
        t = S11.units_mod(v) / q
        s += (round(t) - t) * q
    return s


def main():
    G = {k: v for k, v in QB.groups().items() if len({float(c["q"]) for c in v}) >= 2}
    score: dict = {}
    rng = random.Random(11)
    for k, cells in sorted(G.items()):
        dec = decomps(*k)
        for dn, terms in dec.items():
            ords = list(orderings(terms))
            if PLANT and PLANT.split(":")[0] == dn:
                vals = ords[int(PLANT.split(":")[1])][1]
                for c in cells:
                    c["D"] = 9.0 + psi_sum(vals, float(c["q"])) + rng.uniform(-1, 1)
            for lab, vals in ords:
                R = [float(c["D"]) - psi_sum(vals, float(c["q"])) for c in cells]
                key = (dn, lab)
                score[key] = score.get(key, 0) + (max(R) - min(R) <= TOL)
    print(f"# {len(G)} groups, tol {TOL}" + (f", planted {PLANT}" if PLANT else ""))
    best = sorted(score.items(), key=lambda kv: -kv[1])
    for dn in ("L3", "L4", "L5", "L6", "G4", "G2", "K1"):
        top = [kv for kv in best if kv[0][0] == dn][:3]
        print(dn, " | ".join(f"{s} {lab}" for (_, lab), s in top))


if __name__ == "__main__":
    np.seterr(all="ignore")
    main()

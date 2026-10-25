"""Orders of a log-form exponent in the clean region (H ~ 0), every addition stored (s12's scoring, imported):
  form x  (P = fl(c LN x)):  side 1  [aLNm, bLNn, -hLNn, -lnB...]   side 2  [aLNm, bLNn, -hLNm, -lnB...]
  form mx (P = fl(c LN(fl(m x)))): side 1 [bLNn, -hLNn, -lnB...]   side 2 [bLNn, -lnB...] (c = -b: -b LN(mx))
with -lnB as G2 = fl(fl(GL(h) - GL(a)) - GL(b)) or the three GAMMALNs. Usage: s14_logform.py [--tol 3.5]"""

from __future__ import annotations

import sys

import qbits as QB
import s11_qsearch as S11
import s12_orders as S12

TOL = float(sys.argv[sys.argv.index("--tol") + 1]) if "--tol" in sys.argv else 3.5
LDB, st = QB.LDB, QB.st


def decomps(m: float, n: float, side: int, form: str) -> dict:
    p = S11.pieces(m, n, side)
    G2 = st(LDB(st(LDB(p["gh"]) - LDB(p["ga"]))) - LDB(p["gb"]))
    G3 = [p["gh"], -p["ga"], -p["gb"]]
    if form == "x":
        base = [p["aLNm"], p["bLNn"], -(p["hLNn"] if side == 1 else p["hLNm"])]
    else:
        base = [p["bLNn"], -p["hLNn"]] if side == 1 else [p["bLNn"]]
    return {"G2": base + [G2], "G3": base + G3}


def main():
    for form in ("x", "mx"):
        G = {k: v for k, v in QB.groups(form=form).items() if len({float(c["q"]) for c in v}) >= 2}
        score: dict = {}
        for k, cells in sorted(G.items()):
            for dn, terms in decomps(*k, form).items():
                for lab, vals in S12.orderings(terms):
                    R = [float(c["D"]) - S12.psi_sum(vals, float(c["q"])) for c in cells]
                    score[(dn, lab)] = score.get((dn, lab), 0) + (max(R) - min(R) <= TOL)
        best = sorted(score.items(), key=lambda kv: -kv[1])[:6]
        print(f"form {form}: {len(G)} groups; best " + " | ".join(f"{s} {dn}{lab}" for (dn, lab), s in best))


if __name__ == "__main__":
    main()

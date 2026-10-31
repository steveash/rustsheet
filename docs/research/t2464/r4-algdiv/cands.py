"""T-3383 r4-algdiv: score fixed lnB candidates (no per-shape freedom) for min < 1 < 8 < max under model M2, per
shape and per row, on this worker's probes. A candidate L(a, b) hits a shape when it equals the shape's pinned lnB
(scan.py's best j, unique). Candidates:
  toms_d   r2-bratio's lnb_base (TOMS algdiv in doubles; the frozen choice)
  cr_lnb   the correctly rounded lnB
  cr_alg   fl(GL(a) + CR(algdiv))     GL = Excel's GAMMALN
  gl_ext   fl(GL(a) + fl(GL(b) - GL(a+b)))   the min < 1, max <= 8 GL form carried past 8
  gl_flat  fl(fl(GL(a) + GL(b)) - GL(a+b))
Usage: cands.py a|hold"""

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r3-m1-residual"))
import m1lib as M  # noqa: E402
from lscan import scan  # noqa: E402
from scan import accurate, rows_of  # noqa: E402
from mkgl import gl_values  # noqa: E402

GL = M.BE.GAMMALN
sx = M.sx


def cands(a, b):
    lnb, alg = accurate(a, b)
    return dict(
        toms_d=M.lnb_base(a, b),
        cr_lnb=float(lnb),
        cr_alg=sx("+", GL(a), float(alg)),
        gl_ext=sx("+", GL(a), sx("-", GL(b), GL(a + b))),
        gl_flat=sx("-", sx("+", GL(a), GL(b)), GL(a + b)),
    )


def main(name, W=16):
    rows, _ = rows_of(name)
    M.BE._GL.update(gl_values())  # Excel's GAMMALN at a, b, a + b (probe t3383-r4-algdiv-gl)
    res = scan(rows, "m2", W)
    tot = {}
    for (a, b), (cnt, hits, L0) in res.items():
        for k, L in cands(a, b).items():
            j = M.ulp_steps(L0, L)
            h = hits.get(j, 0) if abs(j) <= W else None
            t = tot.setdefault(k, [0, 0, 0])
            t[0] += h or 0
            t[1] += cnt
            t[2] += h == max(hits.values())
    for k, (h, n, s) in tot.items():
        print(f"{name} {k:8s} rows {h:5d} of {n}   shapes at the pinned lnB {s} of {len(res)}")


if __name__ == "__main__":
    main(sys.argv[1])

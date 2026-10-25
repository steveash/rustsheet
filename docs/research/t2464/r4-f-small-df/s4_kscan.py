"""Scan Excel's constant per (shape, side) on probe mya: F = fl(EXP(K_exact + delta + A)/x), A = M-xd's
fl(c LN x) - h L(r); delta in units (2^-53, absolute in E) on a grid; report the best delta interval and its count.
A group whose best count is the whole group means M-xd's x-structure is exact there and pins Excel's K.
--control: synthetic Excel values from a planted delta per group (recovered?).  Usage: s4_kscan.py [--control] [--set NAME] [maxd]"""
import sys

import numpy as np

import slib as L

ctl = "--control" in sys.argv
SET = sys.argv[sys.argv.index("--set") + 1] if "--set" in sys.argv else "mya"
args = [a for a in sys.argv[1:] if not a.startswith("--") and a != SET]
maxd = float(args[0]) if args else 10
rows = [r for r in L.load(SET) if max(r["d1"], r["d2"]) <= maxd]
g: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], r["d1"], r["d2"])
    g.setdefault((r["d1"], r["d2"], s), []).append(r)
grid = np.arange(-16, 16.001, 0.05)
U = L.LDB(2.0) ** -53
rng = np.random.default_rng(5)
for (m, n, s), rs in sorted(g.items()):
    K = L.ld(L.K_exact(m, n, s))
    A = np.array([L.pieces_A(r["x"], m, n) for r in rs], dtype=L.LDB)
    X = np.array([r["x"] for r in rs], dtype=L.LDB)
    if ctl:
        plant = float(rng.uniform(-8, 8))
        ex = np.array([L.st(L.LDB(L.EXP(K + L.LDB(plant) * U + a)) / x) for a, x in zip(A, X)])
    else:
        ex = np.array([r["excel"] for r in rs])
    cnt = []
    for d in grid:
        E = K + L.LDB(d) * U + A
        v = (L.exp_v6_vec(E).astype(L.LDB) / X).astype(np.float64)
        cnt.append(int((v == ex).sum()))
    cnt = np.array(cnt)
    best = cnt.max()
    sel = grid[cnt == best]
    tag = f" planted {plant:+.2f}" if ctl else ""
    print(f"({m:g},{n:g}) s{s} n {len(rs):3d} best {best:3d} delta [{sel.min():+6.2f}, {sel.max():+6.2f}]"
          f" (cells at delta 0: {cnt[np.argmin(abs(grid))]:3d}){tag}")

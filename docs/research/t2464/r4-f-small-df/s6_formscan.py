"""Free-constant scan per (shape, side) on probe mya for each power-term form (slib.chain): best count of exact
matches over delta (units, absolute in E) added to the exact K. Usage: s6_formscan.py maxd form [form ...]"""
import sys

import numpy as np

import slib as L

maxd = float(sys.argv[1])
forms = sys.argv[2:]
rows = [r for r in L.load("mya") if max(r["d1"], r["d2"]) <= maxd]
g: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], r["d1"], r["d2"])
    g.setdefault((r["d1"], r["d2"], s), []).append(r)
grid = np.arange(-16, 16.001, 0.05)
U = L.LDB(2.0) ** -53
tot = {f: 0 for f in forms}
print("shape side n | " + " | ".join(f"{f}: best [delta]" for f in forms))
for (m, n, s), rs in sorted(g.items()):
    ex = np.array([r["excel"] for r in rs])
    X = np.array([r["x"] for r in rs], dtype=L.LDB)
    out = []
    for f in forms:
        ch = [L.chain(r["x"], m, n, f) for r in rs]
        A = np.array([c[0] for c in ch], dtype=L.LDB)
        K = L.ld(ch[0][1])
        div = ch[0][2]
        cnt = []
        for d in grid:
            v = L.exp_v6_vec(K + L.LDB(d) * U + A)
            if div:
                v = (v.astype(L.LDB) / X).astype(np.float64)
            cnt.append(int((v == ex).sum()))
        cnt = np.array(cnt)
        sel = grid[cnt == cnt.max()]
        tot[f] += cnt.max()
        out.append(f"{f}: {cnt.max():2d} [{sel.min():+.2f},{sel.max():+.2f}]")
    print(f"({m:g},{n:g}) s{s} {len(rs)} | " + " | ".join(out))
print("TOTAL", tot)

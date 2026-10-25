"""Final-step variants on M-xd's E = K + fl(c LN x) - h L(r) (exact K): 'div' fl(EXP(E)/x) (M-xd), 'divx' EXP kept
extended then / x rounded once, 'mulr' fl(EXP(E) * fl(1/x)), 'mulrx' EXP extended * fl(1/x), 'divx64' EXP extended,
/x to 64 bits, then double. Exact-match counts per (shape, side) on mya, max(d) <= maxd. Usage: s9_fin.py maxd"""
import sys

import slib as L

LDB, st = L.LDB, L.st
maxd = float(sys.argv[1])
rows = [r for r in L.load("mya") if max(r["d1"], r["d2"]) <= maxd]


def E_of(x, m, n):
    _, K, _ = L.chain(x, m, n, "xd")
    return L.ld(K) + L.pieces_A(x, m, n)


V = {
    "div": lambda E, x: st(LDB(L.EXP(E)) / LDB(x)),
    "divx": lambda E, x: st(L.R3.exp_v6(E, ext=True) / LDB(x)),
    "mulr": lambda E, x: st(LDB(L.EXP(E)) * LDB(st(LDB(1) / LDB(x)))),
    "mulrx": lambda E, x: st(L.R3.exp_v6(E, ext=True) * LDB(st(LDB(1) / LDB(x)))),
}
g: dict = {}
for r in rows:
    s, _ = L.side_r(r["x"], r["d1"], r["d2"])
    E = E_of(r["x"], r["d1"], r["d2"])
    t = g.setdefault((r["d1"], r["d2"], s), {k: 0 for k in list(V) + ["CR", "n"]})
    for k, f in V.items():
        t[k] += f(E, r["x"]) == r["excel"]
    t["CR"] += r["cr"] == r["excel"]
    t["n"] += 1
tot = {}
for key, t in sorted(g.items()):
    print(f"({key[0]:g},{key[1]:g}) s{key[2]} " + " ".join(f"{k} {v:2d}" for k, v in t.items()))
    for k, v in t.items():
        tot[k] = tot.get(k, 0) + v
print("TOTAL", tot)

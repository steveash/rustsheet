"""Per cell, the set of E offsets (units, absolute, relative to the exact E* = ln(F x) for 'div' finals or ln F for
'none') that reproduce Excel under the final step, by a fine scan; next to M-xd's own E offset.
Usage: s11_einv.py d1 d2 tag [fin]   (fin: div | mulr | none)"""
import sys

import numpy as np

import slib as L

m, n, tag = float(sys.argv[1]), float(sys.argv[2]), sys.argv[3]
fin = sys.argv[4] if len(sys.argv) > 4 else "div"
rows = [r for r in L.load("mya") if (r["d1"], r["d2"]) == (m, n) and r["blk"].endswith(tag + ")")]
U = L.LDB(2.0) ** -53
grid = np.arange(-12, 12.0001, 0.01).astype(L.LDB)
for r in rows:
    x = r["x"]
    X = L.LDB(x)
    Es = L.mpmath.log(r["exact"] * (x if fin != "none" else 1))
    E0 = L.ld(Es)
    v = L.exp_v6_vec(E0 + grid * U)
    if fin == "div":
        v = (v.astype(L.LDB) / X).astype(np.float64)
    elif fin == "mulr":
        v = (v.astype(L.LDB) * L.LDB(L.st(L.LDB(1) / X))).astype(np.float64)
    ok = grid[v == r["excel"]]
    A, K, _ = L.chain(x, m, n, "xd")
    em = float((L.mpf((L.ld(K) + A).as_integer_ratio()[0]) / (L.ld(K) + A).as_integer_ratio()[1] - Es) / L.U)
    iv = f"[{float(ok.min()):+6.2f},{float(ok.max()):+6.2f}]" if len(ok) else "[ none ]"
    inside = len(ok) and ok.min() <= em <= ok.max()
    print(f"{x!r:24s} Excel E in {iv}  M-xd E {em:+6.2f} {'ok' if inside else '--'}")

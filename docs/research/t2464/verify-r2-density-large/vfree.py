"""Free-constant test: per block, lc = st(Cs + c + X); c (units of 2^-53, absolute) fitted on one half (verifier's
sha1 split), exact count scored on the other half.  Compares x87 bd0 against exact bd0 at the same pieces.
Usage: vfree.py arith=x87|exact  sel=cut|r1runs"""
import sys, os, math
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sel = [a for a in sys.argv[1:] if a.startswith("sel=")][0][4:]
sys.argv = [sys.argv[0]] + [a for a in sys.argv[1:] if not a.startswith("sel=")]
import vscore as V, excelmath as XM, mpmath
from collections import defaultdict
mpf = mpmath.mpf
def only(r):
    if sel == "cut":
        return r["src"] == "p1" and int(r["cell"].split("!A")[1]) <= 3072
    return r["src"].startswith("w2:t2464-w2f-xruns") or (r["src"] == "pr1" and r["a"] >= 4)
U = V.main(only)
blocks = defaultdict(list)
for r in U:
    key = (r["src"], r["a"], r["b"], int(r["cell"].split("!A")[1]) // 256 if sel == "cut" else round(r["x"], 2) if r["src"] != "pr1" else "all")
    blocks[key].append(r)
grid = [i * 0.25 for i in range(-48, 49)]
U2 = 2.0 ** -53
for key, rs in sorted(blocks.items(), key=str):
    if len(rs) < 100: continue
    Es = []
    for c in grid:
        for r in rs:
            sh = r["_sh"]
            lc = V.st(sh["Cs"] + mpf(c) * U2 + r["_X"])
            Es.append(float(V.st(sh["L5"] + V.st(lc - sh["lf"] / 2))))
    ex = XM.excel("EXP", Es)
    hits = defaultdict(lambda: [0, 0])
    for gi, c in enumerate(grid):
        for j, r in enumerate(rs):
            hits[c][V.vhold(r)] += ex[gi * len(rs) + j] == r["excel"]
    n0 = sum(1 for r in rs if V.vhold(r) == 0); n1 = len(rs) - n0
    c0 = max(grid, key=lambda c: (hits[c][0], -abs(c)))
    c1 = max(grid, key=lambda c: (hits[c][1], -abs(c)))
    print(f"{V.ARITH:5s} {str(key):55s} fit0 c={c0:+.2f} hold1 {hits[c0][1]}/{n1} | fit1 c={c1:+.2f} hold0 {hits[c1][0]}/{n0} | c=0 {sum(hits[0.0])}/{len(rs)}", flush=True)

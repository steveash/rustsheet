"""Stored-E test in the clean region: E = fl(K' + C + P) (one rounding of the whole exponent at P's grid), C one free
constant per group: is there a C with every |D - (C - psi_q(K' + C))| within TOL/2? Prints feasible C ranges.
Usage: s13_storedE.py [TOL]"""
import sys

import qbits as QB
import s11_qsearch as S11

tol = float(sys.argv[1]) if len(sys.argv) > 1 else 3.5
G = {k: v for k, v in QB.groups().items() if len({float(c["q"]) for c in v}) >= 2}
nok = 0
for k, cs in sorted(G.items()):
    K = float(QB.L.K_exact(*k))
    Km = S11.units_mod(K)
    Kfrac = float((QB.L.K_exact(*k) - QB.mpf(K)) / QB.U)
    ok = []
    for i in range(-2400, 2401):
        C = i / 4
        ph = Km + Kfrac + C
        R = [float(c["D"]) - (C + (round(ph / float(c["q"])) - ph / float(c["q"])) * float(c["q"])) for c in cs]
        if max(R) - min(R) <= tol and abs(sum(R) / len(R)) <= tol / 2:
            ok.append(C)
    nok += bool(ok)
    print(f"({k[0]:g},{k[1]:g}) s{k[2]} C feasible: {ok[:1]}..{ok[-1:]} ({len(ok)})")
print("groups with a feasible C:", nok, "of", len(G))

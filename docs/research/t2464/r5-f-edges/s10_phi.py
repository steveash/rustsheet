"""Feasible phases phi = Q mod max(q) (units) per clean-region group: phi values on a 1/4-unit grid whose residual
spread is within the tolerance, printed as intervals, with the implied C = D + psi_q(phi). Usage: s10_phi.py [TOL]"""
import sys

import sys

import qbits as QB

tol = float(sys.argv[1]) if len(sys.argv) > 1 else 3.0
FORM = next((a.split("=")[1] for a in sys.argv if a.startswith("form=")), "x")
G = QB.groups(form=FORM)
for k, cs in sorted(G.items()):
    qs = sorted({float(c["q"]) for c in cs})
    if len(qs) < 2:
        continue
    qm = qs[-1]
    ok = []
    for i in range(int(qm * 4)):
        phi = QB.mpf(i) / 4 * QB.U
        if QB.spread(cs, phi) <= tol:
            ok.append(i / 4)
    ivs = []
    for v in ok:
        if ivs and v - ivs[-1][1] <= 0.25:
            ivs[-1][1] = v
        else:
            ivs.append([v, v])
    if ok:
        mid = ok[len(ok) // 2]
        C = [float(c["D"] - QB.pred(QB.mpf(mid) * QB.U, c["q"])) for c in cs]
        Cm = sum(C) / len(C)
    print(f"({k[0]:g},{k[1]:g}) s{k[2]} q={qs[0]:g}..{qm:g} phi in " + ", ".join(f"[{a:g},{b:g}]" for a, b in ivs) + (f"  C~{Cm:.1f}" if ok else ""))

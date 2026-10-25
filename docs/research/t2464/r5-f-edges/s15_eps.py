"""Q = K' + eps: from the feasible phases (s10's scan, imported logic) compute eps = (phi - K' mod 1024) wrapped to
(-512, 512] per group, with M-sd's delta for comparison. Usage: s15_eps.py [TOL]"""
import sys

import edgemodel as MD
import qbits as QB
import s11_qsearch as S11

tol = float(sys.argv[1]) if len(sys.argv) > 1 else 3.0
G = {k: v for k, v in QB.groups().items() if len({float(c["q"]) for c in v}) >= 2}
for k, cs in sorted(G.items()):
    qm = max(float(c["q"]) for c in cs)
    Kp = QB.L.K_exact(*k)
    Ku = float((Kp / QB.U) % 1024)
    ok = [i / 4 for i in range(int(qm * 4)) if QB.spread(cs, QB.mpf(i) / 4 * QB.U) <= tol]
    if not ok:
        print(f"({k[0]:g},{k[1]:g}) s{k[2]} no phase")
        continue
    eps = sorted({((v - Ku + qm / 2) % qm) - qm / 2 for v in ok})
    lo, hi = eps[0], eps[-1]
    # C under that phase: D = C - psi(Q)
    mid = ok[len(ok) // 2]
    C = sum(float(c["D"] - QB.pred(QB.mpf(mid) * QB.U, c["q"])) for c in cs) / len(cs)
    print(f"({k[0]:g},{k[1]:g}) s{k[2]} qmax={qm:g} eps in [{lo:g},{hi:g}] (mod {qm:g})  delta_sd={MD.SD.delta(*k):6.1f}")

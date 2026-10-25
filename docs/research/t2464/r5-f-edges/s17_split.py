"""The split check on the four underflow runs: with R = Excel/model - 1 on the subnormal-Y cells (a factor applied
after EXP's subnormal store) and N = the same on the normal-Y cells (q = 1024 units), the stored addend
Q = K_sd - R must have psi_1024(Q) = -N. Usage: s17_split.py"""
import statistics

import edgemodel as MD
import edgeprobe as MK
import qbits as QB
import s4_probe as S4
from fractions import Fraction

runs: dict = {}
for o in MK.rows("a"):
    if o["kind"] != "thresh":
        continue
    e, m = S4.excel_of(o), MD.value(o["x"], o["d1"], o["d2"])
    if not isinstance(e, float) or e == 0:
        continue
    Y = Fraction(e) * Fraction(o["x"])
    ratio = float((Fraction(e) / Fraction(m) - 1) * 2**53)
    kind = "sub" if Y < Fraction(2) ** -1022 * Fraction(1, 2**12) else ("norm" if Y >= Fraction(2) ** -1022 else "edge")
    runs.setdefault((o["d1"], o["d2"]), {}).setdefault(kind, []).append(ratio)
for (m, n), d in runs.items():
    R, N = statistics.median(d["sub"]), statistics.median(d["norm"])
    K = QB.L.K_exact(m, n, 1)
    Qu = K / QB.U + MD.SD.delta(m, n, 1) - R
    ph = float(Qu % 1024)
    ph = ph - 1024 if ph > 512 else ph
    print(f"({m:g},{n:g}) R={R:7.1f} (sd {statistics.pstdev(d['sub']):.1f}, n={len(d['sub'])})  N={N:7.1f} (n={len(d['norm'])})"
          f"  psi(K_sd - R)={ph:7.1f}  needed -N={-N:7.1f}  delta_sd={MD.SD.delta(m, n, 1):.1f}")

"""s17: is Excel's power term T_E = c*L_k + est_k - off on a grid G for one offset off?  Per run, the best off per G
(rms distance to the grid over the plateaus), then the implied rounding of c*L_k (in grid units) per plateau."""
import sys
from fractions import Fraction

import numpy as np

import s16_two

src, want = sys.argv[1], sys.argv[2]
m, n, Ls, est, rm = s16_two.plateaus(src, want)
side1 = None
c = Fraction(int(m), 2) if False else None
import cands as C  # noqa: E402

s, _ = C.pieces(1.0, m, n)
cc = Fraction(s["b"]) if len(sys.argv) < 4 else Fraction(sys.argv[3])
v = np.array([float((cc * Fraction(L) * 2**53) % 4096) for L in Ls])
for kG in (6, 7, 8, 9, 10):
    G = 2.0**kG
    best = None
    for off in np.arange(0, G, 0.5):
        t = (v + est - off) / G
        d = (t - np.round(t)) * G
        r = np.sqrt(np.mean(d**2))
        if best is None or r < best[0]:
            best = (r, off, d)
    print(f"G=2^{kG}: rms dist {best[0]:.1f} at off {best[1]}  (uniform would be {G / 12**0.5:.1f})")
    if kG in (8, 9):
        t = (v - best[1] + est) / G
        frac = (v / G) % 1
        print("   frac(cL/G):", " ".join(f"{f:.2f}" for f in frac[:20]))
        print("   T_E - cL (G):", " ".join(f"{(e - best[1]) / G:+.2f}" for e in est[:20]))

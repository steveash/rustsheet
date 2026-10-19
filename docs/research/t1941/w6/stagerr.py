# T-1941 W6: control diagnostic: truth - chain of every stage's pre-rounding value, in ulps of that stage, at a few x.
# Usage: stagerr.py CHAIN TRUTH [CH]
import sys, numpy as np
sys.path.insert(0, sys.path[0])
import os
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "w5", "p5"))
from meter import chain_sites
from synth import read_chain
def nz(f):
    N, D = read_chain(f)
    return np.array(N) / D[0], np.array(D) / D[0]
(N, D), (Nt, Dt) = nz(sys.argv[1]), nz(sys.argv[2])
x = np.linspace(1.5, 1.999, 6)
for ch, C, Ct in (("N", N, Nt), ("D", D, Dt)):
    if len(sys.argv) > 3 and ch != sys.argv[3]:
        continue
    s, _ = chain_sites(C, x)
    t, _ = chain_sites(Ct, x)
    for j in range(len(C) - 2, -1, -1):
        d = (t[("S", j)][0] - s[("S", j)][0]).astype(float) / np.spacing(np.abs(s[("S", j)][1]))
        print("%s.S%d " % (ch, j) + " ".join("%9.1f" % v for v in d))

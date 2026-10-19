# T-1941 W6: synthetic R rows (rext.py format) for the w6t top-scan layout (coordinator's gen_top: 3,000 runs of 4,096
# consecutive doubles; run i from 1.5 + 0.3u (i % 3 == 0) or 1.8 + 0.1995u, u = frac(0.5 + 0.618034 i); 4 runs per
# case kmyst-gam-w6t%04d), simulated from a chain. Usage: synth_top.py CHAIN OUT [nruns]
import sys, numpy as np
sys.path.insert(0, sys.path[0])
import kern
from synth import read_chain, simB
N, D = read_chain(sys.argv[1])
nr = int(sys.argv[3]) if len(sys.argv) > 3 else 3000
with open(sys.argv[2], "w") as o:
    for i in range(nr):
        u = (0.5 + 0.618034 * i) % 1.0
        x0 = 1.5 + 0.3 * u if i % 3 == 0 else 1.8 + 0.1995 * u
        x = np.empty(4096)
        x[0] = x0
        for k in range(1, 4096):
            x[k] = np.nextafter(x[k - 1], np.inf)
        B, _ = simB(N, D, x)
        lo, hi, n = kern.extract(x, B, 5)
        case = "kmyst-gam-w6t%04d" % (i // 4 + 1)
        for k in range(4096):
            o.write("%s %s %s %s %d %s\n" % (x[k].hex(), B[k].hex(), lo[k].hex(), hi[k].hex(), n[k], case))

"""Cross-check of the BINOM pins with the density column C = BETA.DIST(p,1,n+1,FALSE) ~ EXP(n*l + LNB(n)):
per n group the implied lnB_i = ln(C_i) - n*l_i must be constant (to ~1e-15 abs); a wrong pin shows as a jump of
n*ulp(l). Prints the group spread and rows that deviate by > n*ulp(l)/2.  Usage: dens_check.py [names]"""
import math
import sys
from collections import defaultdict

import mpmath

import make_probes as MP
import pin
import r2lib as R


def main(names):
    g = defaultdict(list)
    for nm in names:
        o = R.read_oracle(nm)
        for i, (p, n, k, l) in enumerate(pin.load(nm)):
            c = o.get(f"C{i + 1}")
            if k in ("none", "multi") or not isinstance(c, float) or c <= 0:
                continue
            q = mpmath.log(R.mpf(c)) - n * R.mpf(l)
            g[n].append((float(q), p, l, nm, k))
    bad = 0
    for n in sorted(g):
        qs = sorted(x[0] for x in g[n])
        med = qs[len(qs) // 2]
        dev = [(x[0] - med, x) for x in g[n]]
        tol = 0.5 * n * 2.0**-55
        out = [d for d in dev if abs(d[0]) > tol]
        bad += len(out)
        print(f"n={n:>10} rows {len(g[n]):5d} lnB~{med:+.15f} spread {max(abs(d[0]) for d in dev):.2e} "
              f"tol {tol:.1e} outliers {len(out)}")
        for d, x in out[:10]:
            print(f"     dev {d:+.3e} (= {d / (n * math.ulp(x[2])):+.2f} n*ulp(l)) p={x[1]!r} {x[3]} pinned k={x[4]}")
    print("outliers", bad)


if __name__ == "__main__":
    main(sys.argv[1:] or ["runs", "grid", "hold"])

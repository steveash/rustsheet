"""b1: the df sweeps (probe sw): Excel's error D = ln(Excel) - ln(exact) in units, per sweep, against the GAMMALN
rounding errors as they would enter a log-gamma constant (gh = e(GL(h)), ga = -e(GL(a)), gb = -e(GL(b)); units
absolute) and the exact magnitude of the big pieces.  Prints the regression of D on (gh + ga + gb) per sweep.
Usage: b1_sweep.py [--list SWEEP]"""
import collections
import sys

import mpmath
import numpy as np

import flib as F
import mkprobe

U = mpmath.mpf(2) ** 53


def gle(v):
    return float((mpmath.mpf(F.GL(v)) - mpmath.loggamma(v)) * U)


def main():
    rows = F.with_exact(mkprobe.rows("sw"))
    F.gl_prefetch({v for r in rows for v in (r["d1"] / 2, r["d2"] / 2, (r["d1"] + r["d2"]) / 2)})
    g = collections.defaultdict(list)
    for r in rows:
        m, n = r["d1"], r["d2"]
        D = float((mpmath.log(r["excel"]) - mpmath.log(r["exact"])) * U)
        G = gle((m + n) / 2) - gle(m / 2) - gle(n / 2)
        g[r["blk"]].append((m, n, D, G, float(mpmath.log(r["exact"]))))
    for blk, v in sorted(g.items()):
        a = np.array([t[2:] for t in v])
        c = np.polyfit(a[:, 1], a[:, 0], 1) if a[:, 1].std() > 0 else (np.nan, np.nan)
        r = a[:, 0] - np.polyval(c, a[:, 1]) if a[:, 1].std() > 0 else a[:, 0]
        print(f"{blk:18s} n={len(v)} D sd {a[:, 0].std():7.2f} | G sd {a[:, 1].std():7.2f} | slope {c[0]:+.2f} "
              f"res sd {r.std():7.2f} | corr {np.corrcoef(a[:, 0], a[:, 1])[0, 1]:+.2f}")
        if "--list" in sys.argv and blk == sys.argv[-1]:
            for t in v:
                print("   ", t[0], t[1], f"D {t[2]:+8.2f}  G {t[3]:+8.2f}  lnF {t[4]:.3f}")
    F.save_cache()


if __name__ == "__main__":
    main()

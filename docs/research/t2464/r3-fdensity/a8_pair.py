"""a8: probe a's paired columns.  Per block: Excel's errors (units, against the exact value at each cell's own
argument) of F.DIST (eF), of BETA.DIST at y = 1/(1+d2/(d1 x)) (eW, w5's form) and at y = d1 x/(d1 x+d2) (eD), their
means, sds and correlations.  If F were Excel's beta density times a Jacobian, eF would track eW or eD.
Usage: a8_pair.py"""
import collections

import numpy as np

import flib as F
import mkprobe

mpf = F.mpf


def main():
    rows = F.with_exact(mkprobe.rows("a"))
    g = collections.defaultdict(list)
    for r in rows:
        x, m, n = r["x"], r["d1"], r["d2"]
        mx = m * x
        yw = 1 / (1 + n / mx)
        yd = mx / (mx + n)
        eF = float((mpf(r["excel"]) - r["exact"]) / r["exact"] * 2**53)
        e = []
        for y, v in ((yw, r["beta_w5"]), (yd, r["beta_d"])):
            b = F.beta_exact(y, m / 2, n / 2)
            e.append(float((mpf(v) - b) / b * 2**53) if v else np.nan)
        g[(m, n, r["blk"])].append((eF, e[0], e[1]))
    print("block n | eF mean sd | eW mean sd | eD mean sd | corr(eF,eW) corr(eF,eD)")
    for k, v in sorted(g.items()):
        a = np.array(v)
        ok = ~np.isnan(a).any(axis=1)
        a = a[ok]
        c1 = np.corrcoef(a[:, 0], a[:, 1])[0, 1] if a[:, 1].std() > 0 else np.nan
        c2 = np.corrcoef(a[:, 0], a[:, 2])[0, 1] if a[:, 2].std() > 0 else np.nan
        print(f"{k[0]:g},{k[1]:g} {k[2]:20s} {len(a):3d} | {a[:, 0].mean():7.2f} {a[:, 0].std():6.2f} | "
              f"{a[:, 1].mean():7.2f} {a[:, 1].std():6.2f} | {a[:, 2].mean():7.2f} {a[:, 2].std():6.2f} | {c1:5.2f} {c2:5.2f}")
    F.save_cache()


if __name__ == "__main__":
    main()

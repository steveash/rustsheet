"""Survey of sum-stage rounding regimes (T-1941 W5). For every run and every sum stage S_j = fl(P_j + c_j) that lands
exactly one binade above P_j, demodulate data and c5-sim residuals at
    tie line     f = S'h/(2 ulp S)   (c_j on P_j's grid: round-half-even ties, period 4 in ulp(P))
    parity line  f = S'h/ulp(S)      (c_j half a P-ulp off the grid: +-1/4 errors, period 2)
and aggregate by (stage, binade of P). The tie/parity regime in Excel reads c_j's bit at ulp(P_j)/2."""

import numpy as np, sys, os, collections

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res

agg = collections.defaultdict(list)
for s in PK.segs:
    xx = PK.x[s]
    xm = xx[len(xx) // 2]
    XL = xx.astype(np.longdouble)
    d = PK.resid(xx, PK.b[s])
    d -= d.mean()
    m = PK.resid(xx, res(xx, PK.N, PK.D))
    m -= m.mean()
    w = np.hanning(len(xx))
    for ch, C in (("N", PK.N), ("D", PK.D)):
        st, _, _ = stages(C, xm)
        for j, P, dP, S, dS in st:
            eP = np.frexp(P)[1]
            eS = np.frexp(S)[1]
            if eS != eP + 1:
                continue
            Sv = np.polyval(C.astype(np.longdouble)[j:][::-1], XL)
            out = []
            for div in (2, 1):
                E = np.exp(-2j * np.pi * np.mod(Sv / np.longdouble(div * ulp(S)), 1.0).astype(float)) * w
                out += [abs(np.dot(d, E)) / len(xx) * 4, abs(np.dot(m, E)) / len(xx) * 4]
            # c_j position on P's grid, in units of ulp(P): frac part tells the c5 regime
            phi = (C[j] / ulp(P)) % 1.0
            agg[(ch, j, eP - 1)].append((xm, phi) + tuple(out))
print("stage  Pbinade  runs  x-range         c5 phi | data tie  sim tie | data par  sim par")
for k in sorted(agg):
    a = np.array(agg[k])
    print(
        "%s.S%d   2^%-4d %4d  %.3f-%.3f   %.2f  |  %.3f   %.3f  |  %.3f   %.3f"
        % (
            k[0],
            k[1],
            k[2],
            len(a),
            a[:, 0].min(),
            a[:, 0].max(),
            a[0, 1],
            a[:, 2].mean(),
            a[:, 3].mean(),
            a[:, 4].mean(),
            a[:, 5].mean(),
        )
    )

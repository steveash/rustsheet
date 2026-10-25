"""Concrete approximations of the correction in l = 2r + c, r = fl(a/fl(a+2)), each in IEEE double, final add in
double ('d') or x87 stored ('x' = fl53(fl64(2r + c))). Scores bit-exact vs pinned l per -r bin.
Usage: approx_scan.py [names]"""
import math
import sys

import pin
import r2lib as R
from struct_scan import pieces

LG = [6.666666666666735130e-01, 3.999999999940941908e-01, 2.857142874366239149e-01, 2.222219843214978396e-01,
      1.818357216161805012e-01, 1.531383769920937332e-01, 1.479819860511658591e-01]


def fdlibm_R(z):
    w = z * z
    t1 = w * (LG[1] + w * (LG[3] + w * LG[5]))
    t2 = z * (LG[0] + w * (LG[2] + w * (LG[4] + w * LG[6])))
    return t2 + t1


def fdlibm_R_horner(z):
    s = LG[6]
    for c in reversed(LG[:-1]):
        s = s * z + c
    return z * s


def taylor_S(t2, N):
    s = 1.0 / (2 * N + 3)
    for k in range(N - 1, -1, -1):
        s = s * t2 + 1.0 / (2 * k + 3)
    return s


def x87add(a, b):
    return R.excelmath.x87("+", a, b)


def cands(r):
    t2 = r * r
    r2 = 2 * r
    out = {}
    c = r * fdlibm_R(t2)
    out["fdlibm Lg 2r + r*R"] = (r2, c)
    out["fdlibm Lg horner"] = (r2, r * fdlibm_R_horner(t2))
    for N in (8, 10, 12, 14, 20):
        S = taylor_S(t2, N)
        out[f"taylor N={N} 2r+2r*(t2*S)"] = (r2, r2 * (t2 * S))
        out[f"taylor N={N} 2r+(2r*t2)*S"] = (r2, (r2 * t2) * S)
    S = pieces(r)[0]
    out["exact S rounded, 2r+2r*(t2*S)"] = (r2, r2 * (t2 * S))
    return out


def main(names):
    rows = [x for nm in names for x in pin.load(nm) if x[2] not in ("none", "multi") and x[0] <= 0.375]
    edges = [0, 0.02, 0.05, 0.1, 0.15, 0.2, 0.2308]
    tab = {}
    for p, n, k, l in rows:
        a = -p
        r = a / (a + 2.0)
        j = max(i for i in range(len(edges) - 1) if -r >= edges[i])
        for nm, (h, c) in cands(r).items():
            for mode in ("d", "x"):
                v = h + c if mode == "d" else x87add(h, c)
                t = tab.setdefault(f"{nm} [{mode}]", [[0, 0] for _ in edges[:-1]])
                t[j][1] += 1
                t[j][0] += v == l
    print("%-40s" % "candidate" + "".join("%12s" % f"<{edges[i+1]}" for i in range(len(edges) - 1)) + "  total")
    for nm, c in tab.items():
        tot = sum(x[0] for x in c), sum(x[1] for x in c)
        print("%-40s" % nm + "".join("%12s" % f"{x[0]}/{x[1]}" for x in c) + f"  {tot[0]}/{tot[1]}")


if __name__ == "__main__":
    main(sys.argv[1:] or ["runs", "grid", "hold"])

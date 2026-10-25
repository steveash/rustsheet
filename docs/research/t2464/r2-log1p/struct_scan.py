"""Outer-structure scan: l built from r = fl(a/fl(a+2)) and an exactly-computed, double-rounded correction
S = (atanh(r)-r)/r^3 (or W = atanh(r)/r), with different op orders, all in IEEE double (x87-stored ops differ only on
double-rounding ties). Scores bit-exact against pinned l per -r bin.  Usage: struct_scan.py [names]"""
import math
import sys

import mpmath

import pin
import r2lib as R

mpf = R.mpf


def pieces(r):
    m = mpf(r)
    at = mpmath.atanh(m)
    S = float((at - m) / m**3) if r != 0 else 1 / 3
    W = float(at / m) if r != 0 else 1.0
    Wm1 = float(at / m - 1) if r != 0 else 0.0
    return S, W, Wm1


def variants(r, S, W, Wm1):
    t2 = r * r
    r2 = 2 * r
    return {
        "A exact c, 1 rounding": float(2 * mpmath.atanh(mpf(r))),
        "B 2*(r + r*t2*S)": 2 * (r + (r * t2) * S),
        "B2 2*(r + r*(t2*S))": 2 * (r + r * (t2 * S)),
        "C 2r + 2r*(t2*S)": r2 + r2 * (t2 * S),
        "C2 2r + (2r*t2)*S": r2 + (r2 * t2) * S,
        "D 2r*W (W double)": r2 * W,
        "E 2r + 2r*Wm1": r2 + r2 * Wm1,
        "F r*(2W)": r * (2 * W),
        "G 2r + t2*(2r*S)": r2 + t2 * (r2 * S),
    }


def main(names):
    rows = [x for nm in names for x in pin.load(nm) if x[2] not in ("none", "multi") and x[0] <= 0.375]
    edges = [0, 0.02, 0.05, 0.1, 0.15, 0.2, 0.2308]
    tab = {}
    for p, n, k, l in rows:
        a = -p
        r = a / (a + 2.0)
        vs = variants(r, *pieces(r))
        j = max(i for i in range(len(edges) - 1) if -r >= edges[i])
        for nm, v in vs.items():
            c = tab.setdefault(nm, [[0, 0] for _ in edges[:-1]])
            c[j][1] += 1
            c[j][0] += v == l
    print("%-24s" % "variant" + "".join("%14s" % f"[{edges[i]},{edges[i+1]})" for i in range(len(edges) - 1)) + "     total")
    for nm, c in tab.items():
        tot = sum(x[0] for x in c), sum(x[1] for x in c)
        print("%-24s" % nm + "".join("%14s" % f"{x[0]}/{x[1]}" for x in c) + f"  {tot[0]}/{tot[1]}")


if __name__ == "__main__":
    main(sys.argv[1:] or ["runs", "grid", "hold"])

"""Per row and outer structure: which doubles S = RN(S_exact) + k ulp (k in -K..K) reproduce Excel's pinned l.
A structure is rejected where a row admits no k; the admitted-k sets map Excel's own S rounding error vs r.
Usage: kset.py [names]"""
import math
import sys

import mpmath

import pin
import r2lib as R

mpf = R.mpf
RMODE = 'x'  # r = x87-stored a/(a+2); 'd' for IEEE double
X = R.excelmath.x87
K = 8


def outer(name, r, S):
    t2 = r * r
    r2 = 2 * r
    if name == "2r+2r*(t2*S) d":
        return r2 + r2 * (t2 * S)
    if name == "2r+2r*(t2*S) x":
        return X("+", r2, r2 * (t2 * S))
    if name == "2r+(2r*t2)*S x":
        return X("+", r2, (r2 * t2) * S)
    if name == "2(r+r*(t2*S)) x":
        return 2 * X("+", r, r * (t2 * S))
    if name == "2(r+(r*t2)*S) x":
        return 2 * X("+", r, (r * t2) * S)
    if name == "2r+t2*(2r*S) x":
        return X("+", r2, t2 * (r2 * S))
    if name == "2r+r*(t2*S2) x":  # S2 = 2S (fdlibm-like R = z*(2/3 + ...))
        return X("+", r2, r * (t2 * (2 * S)))
    raise KeyError(name)


NAMES = ["2r+2r*(t2*S) d", "2r+2r*(t2*S) x", "2r+(2r*t2)*S x", "2(r+r*(t2*S)) x", "2(r+(r*t2)*S) x",
         "2r+t2*(2r*S) x", "2r+r*(t2*S2) x"]


def ksets(names, struct):
    out = []
    for nm in names:
        for p, n, k, l in pin.load(nm):
            if k in ("none", "multi") or p > 0.375 or p < 1e-4:
                continue
            r = R.r_of(p, RMODE)
            m = mpf(r)
            S0 = float((mpmath.atanh(m) - m) / m**3)
            ks = [j for j in range(-K, K + 1) if outer(struct, r, R.nb(S0, j)) == l]
            out.append((-r, p, ks))
    return out


if __name__ == "__main__":
    names = sys.argv[1:] or ["runs", "grid", "hold"]
    for st in NAMES:
        ks = ksets(names, st)
        empty = [x for x in ks if not x[2]]
        excl0 = [x for x in ks if x[2] and 0 not in x[2]]
        print(f"{st:20s} rows {len(ks)} no-k {len(empty)} k=0 excluded {len(excl0)}  "
              f"empty at -r: {[round(x[0], 4) for x in empty][:12]}")

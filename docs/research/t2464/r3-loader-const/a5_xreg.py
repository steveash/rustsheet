"""a5: regress the within-shape residual of the X model on the model's own roundings inside bd0's log form.

Residual_i = mid(extended-E interval) - X_i - median over the shape (units 2^-53). Features are the model's rounding
errors (model value minus the exact op on the same inputs), signed as they enter E: coefficient 0 = Excel has the
rounding (as modelled), -1 = Excel does not round there, +1 = Excel rounds twice as much (e.g. a second store).
Extra candidates absent from the model (coefficient 1 = Excel has them): bd0 sides stored, X stored, R's (t + M) store.
Usage: a5_xreg.py sweep|diverse [min_cells]
"""

from __future__ import annotations

import statistics as st
import sys
from collections import defaultdict

import mpmath
import numpy as np

import r3lib as R

mpf = mpmath.mpf
U = mpf(2) ** -53
D = R.L.D


def side(k, M):
    """Rounding errors (entering E with bd0's minus sign) of M, k/M, LN, k*l for bd0(k, M) in its log form."""
    k = mpf(k)
    Mx = mpf(M)
    r = mpf(D(k / Mx))
    l = mpf(R.L.LN(float(r)))
    t = mpf(D(k * l))
    return [
        -k * (r - k / Mx) / r,  # ratio rounding: d t = k * dr / r
        -k * (l - mpmath.log(r)),  # LN rounding
        -(t - k * l),  # product rounding
    ]


def main():
    which = sys.argv[1]
    rows = R.load(which)
    by = defaultdict(list)
    for r in rows:
        by[(r["a"], r["b"])].append(r)
    Y, F = [], []
    for (a, b), rs in by.items():
        k, n, nk = (mpf(v) for v in R.shape_pieces(a, b))
        ref = R.exact_const(a, b)
        loc = []
        for r in rs:
            ei = R.e_interval_ext(r["excel"])
            if ei is None:
                continue
            X = R.xpart_v(r["x"], a, b, "R")
            x = mpf(r["x"])
            q = mpf(1.0 - r["x"])
            M1, M2 = mpf(D(n * x)), mpf(D(n * q))
            t1 = R.t_log(k, M1, "V0")
            t2 = R.t_log(nk, M2, "V0")
            b1, b2 = t1 + (M1 - k), t2 + (M2 - nk)
            f = [
                float(v / U)
                for v in (
                    [-(1 - k / M1) * (M1 - n * x)]
                    + side(k, M1)
                    + [-(1 - nk / M2) * (M2 - n * q)]
                    + side(nk, M2)
                    + [-(mpf(D(b1)) - b1), -(mpf(D(b2)) - b2), mpf(D(X)) - X, -(mpf(D(t1 + M1)) - (t1 + M1)),
                       -(mpf(D(t2 + M2)) - (t2 + M2))]
                )
            ]
            loc.append((float(((ei[0] + ei[1]) / 2 - X - ref) / U), f))
        if len(loc) < 8:
            continue
        m = st.median(v for v, _ in loc)
        fm = np.mean([f for _, f in loc], axis=0)
        for v, f in loc:
            Y.append(v - m)
            F.append(list(np.array(f) - fm))
    Y, F = np.array(Y), np.array(F)
    names = ["p.M", "p.div", "p.ln", "p.mul", "q.M", "q.div", "q.ln", "q.mul", "bd0p.st", "bd0q.st", "X.st",
             "p.t+M", "q.t+M"]
    c, *_ = np.linalg.lstsq(F, Y, rcond=None)
    print(f"{which}: {len(Y)} cells; residual sd {Y.std():.3f} -> {(Y - F @ c).std():.3f}")
    print("   " + ", ".join(f"{n_}={v:+.2f}" for n_, v in zip(names, c)))
    print("   feature sd: " + ", ".join(f"{n_}={s:.2f}" for n_, s in zip(names, F.std(axis=0))))
    for j, n_ in enumerate(names[8:], 8):  # extra candidates one at a time
        cj, *_ = np.linalg.lstsq(F[:, [j]], Y, rcond=None)
        print(f"   alone {n_:8s} {cj[0]:+.2f}  sd -> {(Y - F[:, [j]] @ cj).std():.3f}")
    R.save_cache()


if __name__ == "__main__":
    main()

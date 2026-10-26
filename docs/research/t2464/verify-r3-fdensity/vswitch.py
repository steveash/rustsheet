"""Fact 3 on a set: per run (shape, block of consecutive doubles), the least-squares slopes of D = ln(Excel/exact)
(units 2^-53) on the rounding errors of s1 = fl(1 + fl(n/mx)) and s2 = fl(1 + fl(mx/n)) (ln stored - ln exact, units),
in units of -h; plus paired win/loss of the frozen ratio-xl-e against CR (w15 and within 4 ulp) per run.
usage: vswitch.py SET   (SET as in vscore.py)"""
import sys

import mpmath
import numpy as np

import vlib as V
import vscore as S

U = mpmath.mpf(2) ** 53
LDB = np.longdouble


def es(x, m, n):
    mx = float(LDB(m) * LDB(x))
    out = []
    for num, den, ex in ((n, mx, mpmath.mpf(n) / (mpmath.mpf(m) * x)), (mx, n, mpmath.mpf(m) * x / n)):
        r = float(LDB(num) / LDB(den))
        s = float(LDB(1) + LDB(r))
        out.append(float((mpmath.log(s) - mpmath.log1p(ex)) * U))
    return out


def runs(rows):
    """Split rows into runs: same shape and x within 64 ulps of the previous row's x."""
    out, cur = [], []
    for r in rows:
        if cur and (r["d1"], r["d2"]) == (cur[-1]["d1"], cur[-1]["d2"]) and 0 < r["x"] - cur[-1]["x"] < 64 * np.spacing(
                r["x"]):
            cur.append(r)
        else:
            if len(cur) >= 10:
                out.append(cur)
            cur = [r]
    if len(cur) >= 10:
        out.append(cur)
    return out


def main():
    rows, _ = S.rows_of(sys.argv[1])
    rows.sort(key=lambda r: (r["case"], int(r["cell"][1:])))
    f = V.model("ratio-xl-e")
    tot = np.zeros(4, int)
    for rs in runs(rows):
        m, n = rs[0]["d1"], rs[0]["d2"]
        h = (m + n) / 2
        ex = [V.f_exact(r["x"], m, n) for r in rs]
        D = np.array([float((mpmath.log(r["excel"]) - mpmath.log(e)) * U) for r, e in zip(rs, ex)])
        E = np.array([es(r["x"], m, n) for r in rs])
        A = np.column_stack([E, np.ones(len(rs))])
        c, *_ = np.linalg.lstsq(A, D, rcond=None)
        mx = m * rs[0]["x"]
        rr = min(mx / n, n / mx)
        pred = ("s2" if mx <= n else "s1") if rr > 0.375 else "none"
        w = np.zeros(4, int)
        for r, e in zip(rs, ex):
            cm = V.classify(f(r["x"], m, n), r["excel"])
            cc = V.classify(float(e), r["excel"])
            w += [cm[2] and not cc[2], cc[2] and not cm[2], cm[1] and not cc[1], cc[1] and not cm[1]]
        tot += w
        print(f"{m:g},{n:g} mx/n {mx / n:7.3f} r {rr:.3f} pred {pred:4s} | s1 {c[0] / -h:+.2f}h s2 {c[1] / -h:+.2f}h"
              f" | D sd {D.std():7.2f} -> {(D - A @ c).std():7.2f} | w15 +{w[0]}/-{w[1]} w4 +{w[2]}/-{w[3]}")
    print(f"total model-vs-CR: w15 +{tot[0]}/-{tot[1]}  within4 +{tot[2]}/-{tot[3]}")
    V.save()


if __name__ == "__main__":
    main()

"""Forward power sum with coefficients held as 64-bit extended reciprocals fl64(1/(2k+1)) (a runtime 1.0/(2k+1)
kept in a register) instead of doubles: term = fl53(fl64(p * c64)). Scored on -r > 0.1 rows.  Usage: coef_ext.py"""
import mpmath

import gen_scan as G
import r2lib as R

fl64, fl53, mpf = R.excelmath.fl64, R.excelmath.fl53, R.mpf
M, A_ = G.M, G.A_


def make(which, N=14):
    c64 = [fl64(mpf(1) / (2 * k + 1)) for k in range(N + 1)]

    def f(r):
        t2 = M(r, r)
        p, s = r, None
        for k in range(1, N + 1):
            p = M(p, t2)
            tm = fl53(fl64(mpf(p) * c64[k])) if k in which else M(p, 1.0 / (2 * k + 1))
            s = tm if s is None else A_(s, tm)
        return 2 * A_(r, s)
    return f


if __name__ == "__main__":
    rows = [x for x in G.load_rows() if -x[1] > 0.1]
    for nm, which in (("all ext", set(range(1, 15))), ("k=1 ext", {1}), ("k>=2 ext", set(range(2, 15)))):
        per, half, miss = G.score(make(which), rows)
        print("%-10s even %d/%d odd %d/%d" % (nm, *half[0], *half[1]), flush=True)

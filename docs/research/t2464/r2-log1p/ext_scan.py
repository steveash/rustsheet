"""Power-sum form l = 2*(r + sum_k RN(1/(2k+1)) * r*t2^k) with chosen intermediates held in x87 extended
(fl64, not stored) instead of stored to double. Flags: t2, p (power chain), term, s (accumulator).
Usage: ext_scan.py"""
import itertools

import mpmath

import gen_scan as G
import r2lib as R

fl64, fl53 = R.excelmath.fl64, R.excelmath.fl53
mpf = R.mpf


def make(ext, N=14):
    def st(v, key):
        v = fl64(v)
        return v if key in ext else mpf(fl53(v))

    cs = [mpf(1.0 / (2 * k + 1)) for k in range(N + 1)]

    def f(r):
        r = mpf(r)
        t2 = st(r * r, "t2")
        p = r
        s = None
        for k in range(1, N + 1):
            p = st(p * t2, "p")
            tm = st(p * cs[k], "term")
            s = tm if s is None else st(s + tm, "s")
        v = st(r + s, "rs")
        return fl53(fl64(2 * v))
    return f


if __name__ == "__main__":
    rows = G.load_rows()
    print("%-30s" % "extended" + "".join("%12s" % f"<{e}" for e in G.EDGES[1:]) + "      even        odd")
    keys = ["t2", "p", "term", "s", "rs"]
    for n in range(0, 3):
        for ext in itertools.combinations(keys, n):
            per, half, miss = G.score(make(set(ext)), rows)
            print("%-30s" % (",".join(ext) or "none") + "".join("%12s" % f"{a}/{b}" for a, b in per)
                  + "".join("%11s" % f"{a}/{b}" for a, b in half), flush=True)

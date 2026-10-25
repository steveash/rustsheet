"""Like ext_scan.make, but the accumulated tail s is stored to double before r + s ('sstore'), with chosen
intermediates extended. Usage: ext_scan3.py"""
import itertools

import ext_scan as E
import gen_scan as G
import r2lib as R

fl64, fl53, mpf = R.excelmath.fl64, R.excelmath.fl53, R.mpf


def make(ext, N=14, order="fwd"):
    def st(v, key):
        v = fl64(v)
        return v if key in ext else mpf(fl53(v))

    cs = [mpf(1.0 / (2 * k + 1)) for k in range(N + 1)]

    def f(r):
        r = mpf(r)
        t2 = st(r * r, "t2")
        p = r
        ts = []
        for k in range(1, N + 1):
            p = st(p * t2, "p")
            ts.append(st(p * cs[k], "term"))
        if order == "rev":
            ts = ts[::-1]
        s = ts[0]
        for tm in ts[1:]:
            s = st(s + tm, "s")
        s = mpf(fl53(s))
        return fl53(fl64(2 * fl53(fl64(r + s))))
    return f


if __name__ == "__main__":
    rows = [x for x in G.load_rows() if -x[1] > 0.1]
    for order in ("fwd", "rev"):
        for n in range(0, 5):
            for ext in itertools.combinations(["t2", "p", "term", "s"], n):
                if "s" not in ext and n > 0 and order == "fwd" and False:
                    continue
                per, half, miss = G.score(make(set(ext), order=order), rows)
                print("%-4s %-20s even %d/%d odd %d/%d" % (order, ",".join(ext) or "none", *half[0], *half[1]), flush=True)

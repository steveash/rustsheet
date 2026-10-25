"""Variants of the best gen_scan family ('fwdtail': l = 2*(r + sum_{k>=1} r^(2k+1)/(2k+1)), terms summed largest
first). Each option is one switch; scored as gen_scan.  Usage: tail_scan.py"""
import itertools

import gen_scan as G
from x87f import x87 as X

M, A_, D_ = G.M, G.A_, G.D_


def make(termop, outer, start, N=14, stop=None):
    def f(r):
        t2 = M(r, r)
        p = r if start == "r" else None
        s = None
        for k in range(1, N + 1):
            p = M(p if p is not None else r, t2) if start == "r" else (M(r, t2) if k == 1 else M(p, t2))
            if termop == "div":
                tm = D_(p, float(2 * k + 1))
            elif termop == "mulrecip":
                tm = M(p, 1.0 / (2 * k + 1))
            else:  # 'rdiv': r * (t2^k / (2k+1)) with q = t2^k tracked
                raise KeyError
            if stop is not None and s is not None and abs(tm) <= stop * abs(s):
                break
            s = tm if s is None else A_(s, tm)
        if outer == "2(r+s)":
            return 2 * A_(r, s)
        if outer == "2r+2s":
            return A_(2 * r, 2 * s)
        if outer == "(r+s)+(r+s)":
            v = A_(r, s)
            return A_(v, v)
        raise KeyError(outer)
    return f


if __name__ == "__main__":
    rows = G.load_rows()
    print("%-36s" % "variant" + "".join("%12s" % f"<{e}" for e in G.EDGES[1:]) + "      even        odd")
    for termop, outer in itertools.product(("div", "mulrecip"), ("2(r+s)",)):
        for stop in (None, 1e-17, 1e-16, 2.2e-16):
            f = make(termop, outer, "r", stop=stop)
            per, half, _ = G.score(f, rows)
            nm = f"{termop} {outer} stop={stop}"
            print("%-36s" % nm + "".join("%12s" % f"{a}/{b}" for a, b in per) + "".join("%11s" % f"{a}/{b}" for a, b in half))

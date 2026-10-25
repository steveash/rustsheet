"""Scan single-coefficient offsets (in ulps) of c_k = RN(1/(2k+1)) in the forward power-sum form (tail_scan2
'fwd chain 2(r+s)'), tuned on the even rows, reported on even and odd.  Usage: coef_scan.py k [range]"""
import sys

import gen_scan as G
import r2lib as R

M, A_ = G.M, G.A_


def make(cs, N=14):
    def f(r):
        t2 = M(r, r)
        p = r
        s = None
        for k in range(1, N + 1):
            p = M(p, t2)
            tm = M(p, cs[k])
            s = tm if s is None else A_(s, tm)
        return 2 * A_(r, s)
    return f


BASE = [1.0 / (2 * k + 1) for k in range(15)]

if __name__ == "__main__":
    k = int(sys.argv[1])
    w = int(sys.argv[2]) if len(sys.argv) > 2 else 8
    rows = [x for x in G.load_rows() if -x[1] > 0.1]
    for j in range(-w, w + 1):
        cs = list(BASE)
        cs[k] = R.nb(cs[k], j)
        per, half, _ = G.score(make(cs), rows)
        print(f"c{k} {j:+3d} ulp: even {half[0][0]}/{half[0][1]} odd {half[1][0]}/{half[1][1]}", flush=True)

"""s8: like s6, but the extra candidates are Excel LN's own errors LN(v) - ln(v) at the stored intermediates v, and
the roundings of the intermediates themselves (relative, units).  A coefficient near +-a, +-b, +-h marks a log
term that Excel takes as LN(v) times a shape.  Usage: s8_lnerr.py SRC MINMAX"""
import sys

import mpmath

import cands as C
import plib as P
import s6_scan

LDB, ex = C.LDB, C.ex
mpf = mpmath.mpf
U = P.U


def extra(x, m, n):
    s, _ = C.pieces(x, m, n)
    X, M, N = mpf(x), mpf(m), mpf(n)
    mx, r1, r2, den = s["mx"], s["r1"], s["r2"], s["den"]
    s1, s2 = C.st(1 + LDB(r1)), C.st(1 + LDB(r2))
    y, q = C.st(LDB(mx) / LDB(den)), C.st(LDB(n) / LDB(den))
    vs = {"mx": (mx, M * X), "r1": (r1, mpf(n) / mpf(mx)), "r2": (r2, mpf(mx) / N), "s1": (s1, 1 + mpf(r1)),
          "s2": (s2, 1 + mpf(r2)), "den": (den, N + mpf(mx)), "y": (y, mpf(mx) / mpf(den)), "q": (q, N / mpf(den))}
    out = {}
    for k, (v, e) in vs.items():
        out[f"eLN({k})"] = float((C.ex(C.LNx(v)) - mpmath.log(mpf(v))) * U)
        out[f"rnd({k})"] = float((mpf(v) - e) / e * U)
    return out


def main():
    src, minmax = sys.argv[1], float(sys.argv[2])
    for blk, rs, m, n in P.runs(src, minmax):
        D, B, base = s6_scan.base_fit(rs, m, n)
        res = s6_scan.scan_extra(D, B, [extra(r["x"], m, n) for r in rs], 0.01)
        print(f"{blk:20s} a={m/2:g} b={n/2:g} base {base:6.1f} | " + " | ".join(f"{k} {sd:.1f} ({c:+.1f})" for sd, k, c in res[:4]))


if __name__ == "__main__":
    main()

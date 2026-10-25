"""Single-regressor search on the needed shifts (s10_need): rounding errors (units) of stored products fl(c * v) for
c in {a, b, h, a-1, b+1, h-1, 1} and v in {LN x, LN mx, LN r, LN(1+r), L(r), LN(n+mx), r, mx}, plus input roundings;
coefficient in {+-1/4 .. +-2}, free offset.  Usage: s14_regs.py SET tag chain"""
import sys

import mpmath
import numpy as np

import r5lib as R
import s10_need as N

LD, LDB, st = R.LD, R.LDB, R.st


def regs(rs, m, n):
    a, b, h = m / 2, n / 2, (m + n) / 2
    out: dict = {}
    for q in rs:
        x = q["x"]
        side1, _, _, _, r, P = R.base(x, m, n)
        mx = st(LDB(m) * LDB(x))
        vs = {"LNx": R.F.LN(x), "LNmx": R.F.LN(mx), "LNr": R.F.LN(r), "LN1r": R.F.LN(st(LDB(1) + LDB(r))),
              "L": LD(R.F.TD.log1p_xl(r)), "LNnmx": R.F.LN(st(LDB(n) + LDB(mx))), "r": LD(r), "mx": LD(mx)}
        for vn, v in vs.items():
            for cn, c in (("a", a), ("b", b), ("h", h), ("a-1", a - 1), ("b+1", b + 1), ("h-1", h - 1)):
                if c == 0:
                    continue
                p, qq = (LD(c) * LD(v)).as_integer_ratio()
                ex = mpmath.mpf(p) / qq
                e = float((mpmath.mpf(st(LD(c) * LD(v))) - ex) * R.U)
                out.setdefault(f"fl({cn}*{vn})", []).append(e)
    return {k: np.array(v) for k, v in out.items()}


if __name__ == "__main__":
    src, tag, ch = sys.argv[1:4]
    for blk, rs, m, n in R.runs(src, lambda m, n, rs: tag in rs[0]["blk"]):
        sc, iv = N.need(rs, m, n, ch)
        lo = np.array([v[0] for v in iv])
        hi = np.array([v[1] for v in iv])
        good = ~np.isnan(lo)
        res = []
        for nm, z0 in regs(rs, m, n).items():
            for co in (-2, -1.5, -1, -0.75, -0.5, -0.25, 0.25, 0.5, 0.75, 1, 1.5, 2):
                z = co * z0
                cnt = max(int(np.sum(good & (lo <= z + o) & (z + o <= hi))) for o in np.arange(-48, 48.1, 1.0))
                res.append((cnt, nm, co))
        res.sort(reverse=True)
        print(blk, ch, "base", sc, "of", len(rs))
        for r_ in res[:12]:
            print("   ", r_)

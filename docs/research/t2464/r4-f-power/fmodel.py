"""T-2586 r4-f-power: frozen F density models (frozen before the hold-out probe was read).

All variants share the x-power found here: P = +fl(a*LN(x)) when mx <= n, -fl(b*LN(x)) when mx > n (Excel's LN, the
product stored), H = -h*L(r) with r = the smaller ratio and L = Excel's log1p (series to 0.375, LN(fl(1+r)) above),
and x^-1 applied apart.  Variants:
  m1     E = K' + P - LN(x) + H, sums extended, F = EXP_V6(E)                          (the ratio model with stored P)
  dd     E = fl(fl(K_hi + H) + P) + K_lo, F = fl(EXP_V6(E) / x)   K_hi = fl(K'), K_lo = K' - K_hi (double-double)
  ddl    as dd with -LN(x) in the extended exponent instead of the division
  g3     as dd with K_hi = fl(fl(fl(Gh - Ga) - Gb) + R) from nearest-double lnGamma pieces
  dd2    as dd, but the first sum's rounding error (K_hi + H) - S1 is carried into the extended exponent
         (S1 = fl(K_hi + H), S2 = fl(S1 + P), E = S2 + lo1 + K_lo); dd2l with -LN(x) in E; dd2g with g3's K_hi
K' = lnGamma(h) - lnGamma(a) - lnGamma(b) + a ln(m/n) (mx <= n) or - b ln(m/n) (mx > n), exact.
Usage: fmodel.py SRC [variant ...]   (SRC: a | hold | r3:a | r3:hold | r3:holdsw | r3:sw)"""
from __future__ import annotations

import collections
import sys


import cands as C
import m1_fixed as M
import m12_khi as M12
import plib as P
import s3_order

LDB, S, ex = C.LDB, C.S, C.ex
F = P.F
R3 = F.R3
VARIANTS = ("CR", "m1", "dd", "ddl", "g3", "dd2", "dd2l", "dd2g")


def value(x: float, m: float, n: float, var: str) -> float:
    s, _ = C.pieces(x, m, n)
    a, b, h = s["a"], s["b"], s["h"]
    side1 = s["mx"] <= n
    c, sg = (a, 1) if side1 else (b, -1)
    Kp = s3_order.kprime(m, n, side1)
    Pp = sg * S(LDB(c) * s["Lx"])
    H = -LDB(h) * M.L_form(s, side1, "xl", n)
    if var == "m1":
        return R3.exp_v6(R3._ld_of(Kp + ex(Pp - s["Lx"] + H)))
    Kh = M12.khi(m, n, side1, "G3" if var in ("g3", "dd2g") else "near")
    lo = Kp - ex(Kh)
    S1 = S(Kh + H)
    if var.startswith("dd2"):  # the first sum's rounding error is carried (TwoSum), the second's is not
        lo = lo + (ex(Kh) + ex(H) - ex(S1))
    E = ex(S(S1 + Pp)) + lo
    if var in ("ddl", "dd2l"):
        return R3.exp_v6(R3._ld_of(E - ex(s["Lx"])))
    return float(R3.exp_v6(R3._ld_of(E), True) / LDB(x))


def main():
    src = sys.argv[1]
    names = sys.argv[2:] or list(VARIANTS)
    rows = P.load(src)
    for nm in names:
        g = collections.defaultdict(list)
        for r in rows:
            v = r["cr"] if nm == "CR" else value(r["x"], r["d1"], r["d2"], nm)
            m, n = r["d1"], r["d2"]
            cls = "min(d)<=2" if min(m, n) <= 2 else ("max(d)>=50" if max(m, n) >= 50 else "other")
            g["all"].append((v, r["excel"]))
            g[cls].append((v, r["excel"]))
        print(f"{src:9s} {nm:5s} " + " | ".join(f"{k} {F.fmt(F.tally(v))}" for k, v in sorted(g.items())))
        sys.stdout.flush()


if __name__ == "__main__":
    main()

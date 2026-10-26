"""Excel's step (change of ln Excel between consecutive x, units) minus the step of a candidate stored power term
P(x) (exact stored-double differences, and the -ln x part exact). If P is Excel's, what is left moves only in the
quanta of the other stored term(s). Usage: s9_resid_steps.py SET run-substring [P forms...]"""
import sys

import mpmath

import fblib as B

F = B.F
LDB, st = F.LDB, F.st
U = mpmath.mpf(2) ** 53
SET, pat = sys.argv[1], sys.argv[2]
forms = sys.argv[3:] or ["xd", "x", "rd"]
rows = B.load(SET)
rr = [r for r in rows if pat in r["blk"]]
m, n = rr[0]["d1"], rr[0]["d2"]
a, b = m / 2, n / 2


def P(x, form):
    mx = st(LDB(m) * LDB(x))
    side = 1 if mx <= n else 2
    if form == "xd":  # fl(c LN x) - ln x (exact)
        c = a if side == 1 else -b
        return mpmath.mpf(st(LDB(c) * F.LN(x))) - mpmath.log(x)
    if form == "x":
        c = (a - 1) if side == 1 else -(b + 1)
        return mpmath.mpf(st(LDB(c) * F.LN(x)))
    if form == "rd":  # fl(c LN r) - ln x
        r = st(LDB(mx) / LDB(n)) if side == 1 else st(LDB(n) / LDB(mx))
        c = a if side == 1 else b
        return mpmath.mpf(st(LDB(c) * F.LN(r))) - mpmath.log(x)
    raise ValueError(form)


dx = [(mpmath.log(q["excel"]) - mpmath.log(p["excel"])) * U for p, q in zip(rr, rr[1:])]
print(f"{rr[0]['blk']}: excel steps", " ".join(f"{float(v):.0f}" for v in dx))
for f in forms:
    dp = [(P(q["x"], f) - P(p["x"], f)) * U for p, q in zip(rr, rr[1:])]
    print(f"  minus {f:4s}:", " ".join(f"{float(v - w):.0f}" for v, w in zip(dx, dp)))

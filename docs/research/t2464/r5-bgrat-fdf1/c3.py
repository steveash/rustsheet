"""T-3567 r5-fdf1 c3: F at d1 = 1 in bgrat (b = 1/2): which ln y does Excel's bgrat take? bgrat_half_ld per lnx form."""
import io, math, sys, contextlib
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "r2-tdist"))
import xlk  # noqa
with contextlib.redirect_stdout(io.StringIO()):
    import c2  # noqa
bx = xlk.bx
K = bx._KERNELS[c2.CFG]
L1P = lambda v: K.alnrel(v)
LN = bx.LN
rows = [(r, v) for r, (v, b) in zip(c2.rows, c2.mv) if b == "bgrat"]
def parts(x, n):
    s = n + x
    xx, y = x / s, n / s
    return xx, y
FORMS = {
    "LN(fl(n/s))": lambda xx, y, x, n: LN(y),
    "LN(fl(1-xx))": lambda xx, y, x, n: LN(1 - xx),
    "log1p(-xx)": lambda xx, y, x, n: L1P(-xx),
    "-log1p(fl(x/n))": lambda xx, y, x, n: -L1P(x / n),
    "-LN(fl(1+fl(x/n)))": lambda xx, y, x, n: -LN(1 + x / n),
    "LN(n)-LN(s)": lambda xx, y, x, n: LN(n) - LN(n + x),
}
for name, f in FORMS.items():
    ex = w4 = 0; errs = []
    for (fn, x, n, up, e), _ in rows:
        xx, y = parts(x, n)
        w1 = xlk.bgrat_half_ld(n / 2, f(xx, y, x, n), 0.0)
        v = w1 if up else 0.5 + (0.5 - w1)
        d = (v - e) / math.ulp(e)
        ex += v == e; w4 += abs(d) <= 4; errs.append(round(d))
    print(f"{name:22s} {ex}/{w4}/{len(rows)}  {errs}")
from fractions import Fraction as Fr
def rn(q): return float(q)  # Fraction -> nearest double
YF = {
    "RN(n/(n+x))": lambda x, n: rn(Fr(n) / (Fr(n) + Fr(x))),
    "1-RN(x/(n+x))": lambda x, n: 1 - rn(Fr(x) / (Fr(n) + Fr(x))),
    "fl(1/fl(1+fl(x/n)))": lambda x, n: 1 / (1 + x / n),
    "fl(1-fl(1/fl(1+fl(n/x))))": lambda x, n: 1 - 1 / (1 + n / x),
    "fl(1-fl(fl(x/n)/fl(1+fl(x/n))))": lambda x, n: 1 - (x / n) / (1 + x / n),
    "fl(1/fl(1+fl(x/n))) via r=n/x: fl(r/fl(1+r))": lambda x, n: (n / x) / (1 + n / x),
    "fl(1-fl(1/fl(1+fl(n/x)))) ": lambda x, n: 1 - 1 / (1 + n / x),
    "fl(0.5+(0.5-xx))": lambda x, n: 0.5 + (0.5 - x / (n + x)),
}
print("--- lnx = LN(y) per y form")
for name, f in YF.items():
    ex = w4 = 0; errs = []
    for (fn, x, n, up, e), _ in rows:
        y = f(x, n)
        w1 = xlk.bgrat_half_ld(n / 2, LN(y), 0.0)
        v = w1 if up else 0.5 + (0.5 - w1)
        d = (v - e) / math.ulp(e)
        ex += v == e; w4 += abs(d) <= 4; errs.append(round(d))
    print(f"{name:40s} {ex}/{w4}/{len(rows)}  {errs}")

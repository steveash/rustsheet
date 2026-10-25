import math, sys
from fractions import Fraction
sys.path.insert(0, '.')
import w5lib as L
import mpmath
for c in L.cells({"BINOM.DIST"}):
    a = c["args"]
    if len(a) < 4 or not isinstance(a[3], bool): continue
    k, n, p, cum = math.trunc(a[0]), math.trunc(a[1]), a[2], a[3]
    ex = c["excel"]
    if k == 0:
        q = 1 - p
        m = L.EXP(n * L.LN(q))
        m2 = L.EXP(n * float(mpmath.log1p(-p)))
        m3 = L.EXP(L.x87('*', n, L.LN(q)))
        if m != ex:
            print(f"k=0 {'cdf' if cum else 'pmf'} n={n} p={p} q={q!r} ex={ex!r} EXP(nLNq) {(m-ex)/math.ulp(ex):+.0f} EXP(n log1p) {(m2-ex)/math.ulp(ex):+.0f} x87 {(m3-ex)/math.ulp(ex):+.0f} exact {(float(Fraction(q)**n)-ex)/math.ulp(ex):+.1f}")
    if k == n - 1 and cum:
        P = L.EXP(n * L.LN(p)); m = (0.5 - P) + 0.5
        if m != ex:
            print(f"k=n-1 n={n} p={p} ex={ex!r} 1-P {(m-ex)/math.ulp(ex):+.0f}  exact {(float(1-Fraction(p)**n)-ex)/math.ulp(ex):+.1f}")

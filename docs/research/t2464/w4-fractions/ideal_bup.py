"""Idealized tie response of TOMS bratio's L140 path (bup + bpser) under prefactor hypotheses for brcmp1(b0',a0,y0,x0):
 Ha: Excel-pdf style (b0'-1)LN(y0) + (a0-1)LN(1-y0), times y0*x0;  Hb: b0' LN(y0) + a0 LN(1-y0);  H0: no y (TOMS)."""
import mpmath as mp
from ideal_c import EXCEL, I
mp.mp.dps = 40
M = mp.mpf
for a, b, x, c in EXCEL[:5]:
    a, b, x = M(a), M(b), M(x); y = 1 - x
    n = int(b); b0 = b - n
    if b0 == 0: n -= 1; b0 = M(1)
    apb = a + b0; ap1 = b0 + 1
    def W(yy):
        d = M(1); w = M(1)
        for l in range(n - 1):
            d *= (apb + l) / (ap1 + l) * yy; w += d
        return w
    h = M(10) ** -25
    cW = float((mp.log(W(y * (1 + h))) - mp.log(W(y))) / h)
    bup = y ** b0 * x ** a / mp.beta(b0, a) / b0 * W(y)
    tot = I(a, b, x)
    frac = float(bup / tot)
    chk = float((bup + I(a, b0, x)) / tot - 1)
    Ha = (b0 - 1) + 1 - (a - 1) * y / x
    Hb = b0 - a * y / x
    print(f"a={float(a)} b={float(b)} x={float(x)} Excel {c:+.2f} | n={n} b0'={float(b0)} frac_bup={frac:.3f} cW={cW:+.2f} chk {chk:.0e} | "
          f"H0 {frac*cW:+.2f}  Ha {frac*(cW+float(Ha)):+.2f}  Hb {frac*(cW+float(Hb)):+.2f}  Ha+x2 {frac*(cW+float(Ha-y/x)):+.2f}")

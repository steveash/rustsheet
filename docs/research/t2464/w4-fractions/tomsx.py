"""w4-fractions: TOMS 708 bratio structure with Excel's pieces ("TOMS-X").

brcomp(a,b,x,y) and brcmp1 are Excel's density times x times y (finding of RESULTS.md section 3): the density is
w1-density's model M1 for min(a,b) <= 1 (LN(x), LN(fl(1-x)) above x = 0.375 else log1p, stored (a-1)*LN, (b-1)*ly,
extended E = tx + (ty - lnB), EXP model V6), with lnB = fl(GL(min) + fl(GL(max) - GL(a+b))) for max <= 8, and
lnB = -LN(b) for a = 1 (w1).  Variants of every unknown are flags."""
import math, sys
from pathlib import Path
import mpmath
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'w1-density'))
sys.path.insert(0, str(HERE.parent))
import lib as w1  # noqa: E402  (w1-density helpers: exp_v6, R64, D, xl cache)
from common4 import LD, x as x87  # noqa: E402
mpf = mpmath.mpf
_cache = {}
def XL(name, v):
    k = (name, v)
    if k not in _cache:
        _cache[k] = w1.xl(name, [v])[0]
    return _cache[k]
def prefetch(name, vs):
    need = sorted({v for v in vs if (name, v) not in _cache})
    if need:
        for v, r in zip(need, w1.xl(name, need)): _cache[(name, v)] = r

def lnB_small(a, b):
    lo, hi = min(a, b), max(a, b)
    if lo == 1.0 and hi > 1.0:
        return -XL('LN', hi)
    return w1.D(mpf(XL('GAMMALN', lo)) + mpf(w1.D(mpf(XL('GAMMALN', hi)) - mpf(XL('GAMMALN', a + b)))))

def pdf_m1(xx, a, b, lnB=None, ly_mode='m1'):
    """Excel density model M1 (w1-density) at xx for min(a,b) <= 1, max <= 8."""
    L = mpf(lnB if lnB is not None else lnB_small(a, b))
    lnx = mpf(XL('LN', xx))
    if ly_mode == 'm1' and xx <= 0.375:
        ly = mpf(float(mpmath.log1p(-mpf(xx))))
    else:
        ly = mpf(XL('LN', 1.0 - xx))
    tx = mpf(w1.D(mpf(w1.D(mpf(a) - 1)) * lnx))
    ty = mpf(w1.D(mpf(w1.D(mpf(b) - 1)) * ly))
    if b == 1.0:
        E = mpf(w1.D(tx - L))
    else:
        E = w1.R64(tx + w1.R64(ty - L))
    return w1.exp_v6(E)

def bup_l140(a0, b0, x0, y0, order='pxy', wsum='sse'):
    """TOMS L140 bup(b0', a0, y0, x0, n): brcmp1 = pdf(y0; b0', a0)*y0*x0, /b0', times W(y0)."""
    n = int(b0); bb = b0 - n
    if bb == 0: n -= 1; bb = 1.0
    A, B, X, Y = bb, a0, y0, x0   # bup(a=b0', b=a0, x=y0, y=x0)
    P = pdf_m1(X, A, B)
    if order == 'pxy': brc = x87('*', x87('*', P, X), Y)
    elif order == 'p(xy)': brc = x87('*', P, x87('*', X, Y))
    else: brc = P * X * Y
    ret = x87('/', brc, A)
    apb, ap1 = A + B, A + 1.0
    # TOMS: k loop split by r = (b-1)x/y - a; summation identical numerically except the break test
    d = 1.0; w = 1.0
    for l in range(n - 1):
        if wsum == 'sse':
            d = d * ((apb + l) / (ap1 + l) * X); w = w + d
        else:
            d = x87('*', d, x87('*', x87('/', x87('+', apb, float(l)), x87('+', ap1, float(l))), X)); w = x87('+', w, d)
    return x87('*', ret, w), bb

def bpser(a, b, xx, eps=1e-15, pref='expln'):
    """TOMS bpser(a, b, x) for a >= 1 (Excel-style prefactor variants)."""
    lnB = lnB_small(a, b)
    if pref == 'expln':
        z = mpf(w1.D(mpf(w1.D(mpf(a) * mpf(XL('LN', xx)))) - mpf(lnB)))
        ans = x87('/', w1.exp_v6(z), a)
    else:  # extended z
        z = w1.R64(mpf(w1.D(mpf(a) * mpf(XL('LN', xx)))) - mpf(lnB))
        ans = x87('/', w1.exp_v6(z), a)
    tol = eps / a; s = 0.0; c = 1.0; n = 0
    while True:
        n += 1
        c = c * ((0.5 - b / n + 0.5) * xx)
        w = c / (a + n); s += w
        if not (n < 10_000_000 and abs(w) > tol): break
    return ans * (a * s + 1.0)

def l140(xx, a, b, **kw):
    """Direct cdf by TOMS L140 for a, b > 1, b < 40, b*x > 0.7, x <= 0.7 (no swap)."""
    y = 1.0 - xx
    up, bb = bup_l140(a, b, xx, y, **kw)
    return up + bpser(a, bb, xx)

# ---- best L140 assembly (section 4 of RESULTS.md): W with the ratio stored, ratio*y0 extended, d and w stored;
#      bup = fl(fl(P*fl(y0*x))/b0') * W; cdf = fl(bup + bpser)
def lnB_any(a, b):
    import toms_betaln as tb
    if max(a, b) <= 8.0 and min(a, b) <= 1.0:
        return lnB_small(a, b)
    return tb.betaln(a, b)

def W_best(y0, a0, b0, n):
    apb, ap1 = LD(b0 + a0), LD(b0 + 1.0); Y = LD(y0); d = LD(1); w = LD(1)
    for l in range(n - 1):
        num = LD(float(apb + l)); den = LD(float(ap1 + l))     # apb + l, ap1 + l stored (doubles)
        r = LD(float(num / den)); d = LD(float(d * (r * Y))); w = LD(float(w + d))
    return float(w)

def l140_best(xx, a, b, P=None, Q=None):
    """TOMS L140 with Excel pieces modelled (P: w1 M1 density at y0 of Beta(b0', a); Q: bpser model) unless given."""
    y0 = 1.0 - xx
    n = int(b); bb = b - n
    if bb == 0: n -= 1; bb = 1.0
    if P is None: P = pdf_m1(y0, bb, a, lnB=lnB_any(bb, a))
    if Q is None: Q = bpser(a, bb, xx)
    w = W_best(y0, a, bb, n)
    return ((P * (y0 * xx)) / bb) * w + Q

def toms_path(xx, a, b):
    """Which TOMS 708 bratio branch the (unswapped) direct point takes, for a, b > 1."""
    lam = a - (a + b) * xx if a <= b else (a + b) * (1 - xx) - b
    if lam < 0: return 'swap'
    if b < 40:
        return 'bpser' if b * xx <= 0.7 else ('L140' if xx <= 0.7 else 'L140-bgrat')
    if a > b:
        return 'bfrac' if (b <= 100 or lam > 0.03 * b) else 'basym'
    return 'bfrac' if (a <= 100 or lam > 0.03 * a) else 'basym'

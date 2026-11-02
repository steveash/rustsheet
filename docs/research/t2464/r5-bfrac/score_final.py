"""T-3571 r5-bfrac: score the frozen model (model.py) against M0 (TOMS 708 bfrac as the engine ports it: R's order in
doubles, eps 15*1e-15, TOMS lambda, cdf = fl(fl(P*fl(x*y))*r)) and CR (the correctly rounded I_x(a,b)), all on Excel's
own density P. Counts: bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15).
usage: score_final.py [case ...]   (default: tune, hold)"""
import sys, math, mpmath
from runs import load, groups
from exact_r import r_exact
import model
mpmath.mp.dps = 40
M = mpmath.mpf

def ulps(m, e):
    return 0.0 if m == e else abs(m - e) / math.ulp(e)

def w15(m, e):
    if m == e: return True
    k = math.floor(math.log10(abs(e)))
    return abs(m - e) <= 0.5 * 10 ** (k - 14)

def m0(r):
    a, b, x, y = r['a'], r['b'], r['x'], r['y']
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    c = lam + 1.; c0 = b / a; c1 = 1. / a + 1.; yp1 = y + 1.; n = 0.; p = 1.; s = a + 1.
    an, bn, anp1, bnp1 = 0., 1., 1., c / c1; rr = c1 / c
    for _ in range(1000):
        n += 1.; w = n * x * (b - n); t = n / a; e = a / s
        alpha = p * (p + c0) * e * e * (w * x); e = (t + 1.) / (c1 + t + t); beta = w / s + (n + e * (c + n * yp1))
        p = t + 1.; s += 2.
        t = alpha * an + beta * anp1; an = anp1; anp1 = t
        t = alpha * bn + beta * bnp1; bn = bnp1; bnp1 = t
        r0 = rr; rr = anp1 / bnp1
        if abs(rr - r0) <= 1.5e-14 * rr: break
        an /= bnp1; bn /= bnp1; anp1 = rr; bnp1 = 1.
    return (r['pdf'] * (x * y)) * rr

def cr(r):
    X, A, B = M(r['x']), M(r['a']), M(r['b'])
    rx = r_exact(A, B, X, 1 - X, A - (A + B) * X)
    return float(r['px'] * X * (1 - X) * rx)

MODELS = {'FROZEN': lambda r: model.cdf(r['x'], r['a'], r['b'], r['pdf']), 'M0 (TOMS dbl)': m0, 'CR': cr}

def table(rows):
    out = {}
    for nm, fn in MODELS.items():
        ms = [fn(r) for r in rows]
        out[nm] = (sum(m == r['cdf'] for m, r in zip(ms, rows)), sum(ulps(m, r['cdf']) <= 4 for m, r in zip(ms, rows)),
                   sum(w15(m, r['cdf']) for m, r in zip(ms, rows)))
    return out

if __name__ == '__main__':
    for case in sys.argv[1:] or ['t3571-r5-bfrac-tune', 't3571-r5-bfrac-hold']:
        rows = load(case)
        print(f'{case}: {len(rows)} cells')
        for nm, (e, w4, w) in table(rows).items(): print(f'  {nm:14s} {e}/{w4}/{w}')
        for key, rs in groups(rows).items():
            t = table(rs)
            print(f'    {key[0]:g},{key[1]:g} z{key[2]:+g} n={len(rs)} ' + ' | '.join(f'{nm} {e}/{w4}/{w}' for nm, (e, w4, w) in t.items()))

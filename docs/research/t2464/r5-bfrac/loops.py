"""T-3571: bfrac's fraction r = 1/(c/c1 + K(alpha_n/beta_n)) evaluated by different loops in Python doubles (each op
correctly rounded, as SSE2). Scores correlation of each loop's error with Excel's implied fraction error, and exact
counts with cdf = fl(fl(P*fl(x*y))*r). Terms alpha_n, beta_n are TOMS 708's (R's C order)."""
import sys, math, numpy as np, mpmath
from exact_r import get

def terms(a, b, x, y, lam, nmax):
    c = 1.0 + lam; c0 = b / a; c1 = 1.0 + 1.0 / a; yp1 = y + 1.0; n = 0.0; p = 1.0; s = a + 1.0
    out = []
    for _ in range(nmax):
        n += 1.0; t = n / a; w = n * (b - n) * x; e = a / s
        alpha = p * (p + c0) * e * e * (w * x)
        e = (1.0 + t) / (c1 + t + t); beta = n + w / s + e * (c + n * yp1); p = 1.0 + t; s += 2.0
        out.append((alpha, beta))
    return c, c1, out

def toms(a, b, x, y, lam, eps=1.5e-14, rescale=True):
    c, c1, T = terms(a, b, x, y, lam, 400)
    an, bn, anp1, bnp1, r = 0.0, 1.0, 1.0, c / c1, c1 / c
    for k, (al, be) in enumerate(T):
        t = al * an + be * anp1; an = anp1; anp1 = t
        t = al * bn + be * bnp1; bn = bnp1; bnp1 = t
        r0 = r; r = anp1 / bnp1
        if abs(r - r0) <= eps * r: return r, k + 1
        if rescale: an /= bnp1; bn /= bnp1; anp1 = r; bnp1 = 1.0
    return r, -1

def lentz(a, b, x, y, lam, eps=1.5e-14, tiny=1e-300, inv=True):
    c, c1, T = terms(a, b, x, y, lam, 400)
    f = c / c1; C = f; D = 0.0
    for k, (al, be) in enumerate(T):
        D = be + al * D; D = D if D != 0 else tiny
        C = be + al / C; C = C if C != 0 else tiny
        D = 1.0 / D; de = C * D; f = f * de
        if abs(de - 1.0) <= eps: return (1.0 / f if inv else f), k + 1
    return 1.0 / f, -1

def backward(a, b, x, y, lam, N=60):
    c, c1, T = terms(a, b, x, y, lam, N)
    v = 0.0
    for al, be in reversed(T): v = al / (be + v)
    return 1.0 / (c / c1 + v), N

def steed(a, b, x, y, lam, eps=1.5e-14):
    """Steed's algorithm on F = b0 + K(a_n/b_n)."""
    c, c1, T = terms(a, b, x, y, lam, 400)
    b0 = c / c1; al, be = T[0]
    D = 1.0 / be; dh = al * D; h = b0 + dh
    for k, (al, be) in enumerate(T[1:]):
        D = 1.0 / (D * al + be); dh = (be * D - 1.0) * dh; h += dh
        if abs(dh) <= eps * abs(h): return 1.0 / h, k + 2
    return 1.0 / h, -1

def run(rows, fn, **kw):
    U = mpmath.mpf(2) ** -53; M = mpmath.mpf
    err, ex, its = [], 0, []
    for r in rows:
        x = r['x']; y = 1.0 - x; lam = r['a'] - (r['a'] + r['b']) * x
        v, it = fn(r['a'], r['b'], x, y, lam, **kw); its.append(it)
        err.append(float((M(v) / r['rx'] - 1) / U))
        ex += (r['pdf'] * (x * y)) * v == r['cdf']
    return np.array(err), ex, its

if __name__ == '__main__':
    rows = get()
    M = mpmath.mpf; U = M(2) ** -53
    ee = np.array([float((M(r['cdf']) / (M(r['pdf']) * M(r['x'] * (1 - r['x']))) / r['rx'] - 1) / U) for r in rows])
    cands = [('toms', toms, {}), ('toms eps2.2e-16', toms, dict(eps=2.2e-16)), ('toms no rescale', toms, dict(rescale=False)),
             ('lentz 1.5e-14', lentz, {}), ('lentz 1e-15', lentz, dict(eps=1e-15)), ('lentz 2.2e-16', lentz, dict(eps=2.2e-16)),
             ('steed 1.5e-14', steed, {}), ('steed 2.2e-16', steed, dict(eps=2.2e-16))]
    cands += [(f'backward {N}', backward, dict(N=N)) for N in (30, 40, 60, 100)]
    for nm, fn, kw in cands:
        e, ex, its = run(rows, fn, **kw)
        print(f'{nm:18s} exact {ex:4d} mean {e.mean():+.2f} sd {e.std():.2f} corr {np.corrcoef(ee, e)[0,1]:+.3f} '
              f'resid sd {np.std(ee - e):.2f} iters {min(its)}-{max(its)}')

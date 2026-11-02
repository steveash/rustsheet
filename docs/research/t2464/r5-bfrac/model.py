"""T-3571 r5-bfrac: the frozen model of Excel's bfrac branch (BETA.DIST cdf where TOMS 708 bratio takes bfrac).

Excel keeps TOMS 708's continued fraction for I_x(a, b) (Didonato & Morris's bfrac, a, b >= 40 region) but evaluates
it with the modified Lentz method of Numerical Recipes (betacf's loop), on terms written in closed form. In doubles:

  lam = fl(a - fl((a+b)*x))            (both orientations; TOMS takes (a+b)*y - b when a > b)
  y   = fl(1 - x),  c = lam + 1,  b0 = c * (a/(a+1))
  term n = 1, 2, ...:  u = a+n-1, v = a+b+n-1, s = a+2n-1 (exact), m = n*(b-n)
      alpha_n = ALPHA(u, v, s, m, x)
      beta_n  = n + fl(n*x*(b-n))/s + ((a+n)/(a+2n+1)) * (c + n*(y+1))
  r = 0 + 1/(b0 + alpha_1/(beta_1 + ...)) by Lentz from TINY: f = TINY, C = f, D = 0; the partial (1, b0); then for
      each term D = 1/(beta + alpha*D), C = beta + alpha/C, d = C*D, f = f*d, until |d - 1| <= EPS
  cdf = fl(fl(P * fl(x*y)) * f),  P = Excel's own density BETA.DIST(x, a, b, FALSE) (brcomp = density*x*y)

EPS = 10*DBL_EPSILON is FITTED: the scan of stop thresholds (score_eps.py) brackets it in (2.1e-15, 2.5e-15) on the
tune probe; 10*DBL_EPSILON is the natural constant inside the bracket. TINY = 1e-30 (Numerical Recipes' FPMIN; 1e-300
scores lower, score_loop.py). The term forms are structural choices ranked on the tune probe (alpha_enum.py,
beta_enum.py), not fitted constants."""
EPS = 2.220446049250313e-15  # FITTED (2.1e-15, 2.5e-15): 10 * DBL_EPSILON
TINY = 1e-30

def alpha(u, v, s, m, x):
    return ALPHA_FORM(u, v, s, m, x)

ALPHA_FORM = None  # set below at the freeze

def terms(a, b, x, y, lam):
    c = lam + 1.0
    n = 0.0
    while True:
        n += 1.0
        u = a + n - 1.0; v = a + b + n - 1.0; s = a + 2.0 * n - 1.0; m = n * (b - n)
        al = alpha(u, v, s, m, x)
        be = n + n * x * (b - n) / s + (a + n) / (a + 2.0 * n + 1.0) * (c + n * (y + 1.0))
        yield al, be

def fraction(a, b, x, y, lam, eps=EPS, maxit=3000):
    """r = 1/(b0 + K(alpha_n/beta_n)) by Numerical Recipes' modified Lentz loop; returns (r, terms used)."""
    b0 = (lam + 1.0) * (a / (a + 1.0))
    f = TINY; C = f; D = 0.0
    D = 1.0 / (b0 + 1.0 * D); C = b0 + 1.0 / C; f = f * (C * D)
    k = 0
    for k, (al, be) in enumerate(terms(a, b, x, y, lam), 1):
        D = 1.0 / (be + al * D)
        C = be + al / C
        d = C * D; f = f * d
        if abs(d - 1.0) <= eps or k >= maxit: break
    return f, k

def cdf(x, a, b, P):
    """Excel's BETA.DIST(x, a, b, TRUE) on a bfrac-branch point, given Excel's density P there."""
    y = 1.0 - x
    lam = a - (a + b) * x
    return (P * (x * y)) * fraction(a, b, x, y, lam)[0]

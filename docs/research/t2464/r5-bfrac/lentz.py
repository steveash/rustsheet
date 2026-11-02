"""T-3571 r5-bfrac: Excel's bfrac fraction as a modified Lentz loop (Numerical Recipes style) on TOMS 708's
continued fraction, with pluggable term formulas; doubles (Python floats, each op correctly rounded as SSE2).

r = 1/(b0 + K(alpha_n/beta_n)), b0 = c/c1, run as the fraction 0 + 1/(b0 + ...): f = tiny, C = f, D = 0, the first
partial (1, b0), then (alpha_n, beta_n): D = 1/(beta + alpha*D), C = beta + alpha/C, f *= C*D, stop at |C*D - 1| <= eps.
cdf = fl(fl(P*fl(x*y)) * f), P = Excel's own density (BETA.DIST(x, a, b, FALSE))."""
import math
from fractions import Fraction as Fr

def toms_terms(a, b, x, y, lam):
    c = lam + 1.0; c0 = b / a; c1 = 1.0 / a + 1.0; yp1 = y + 1.0; n = 0.0; p = 1.0; s = a + 1.0
    def gen():
        nonlocal n, p, s
        while True:
            n += 1.0; w = n * (b - n) * x; t = n / a; e = a / s
            alpha = p * (p + c0) * e * e * (w * x)
            e = (t + 1.0) / (c1 + t + t); beta = n + w / s + e * (c + n * yp1)
            p = t + 1.0; s += 2.0
            yield alpha, beta
    return c / c1, gen()

def cr_terms(a, b, x, y, lam):
    """Each term correctly rounded from exact arithmetic on the double inputs (an ideal term formula)."""
    A, B, X, Y, L = map(Fr, (a, b, x, y, lam))
    c = L + 1; c1 = 1 / A + 1
    def gen():
        n = 0
        while True:
            n += 1
            al = n * (A + n - 1) * (A + B + n - 1) * (B - n) * X * X / (A + 2 * n - 1) ** 2
            be = n + n * (B - n) * X / (A + 2 * n - 1) + (A + n) * (c + n * (1 + Y)) / (A + 2 * n + 1)
            yield float(al), float(be)
    return float(c / c1), gen()

def lentz(a, b, x, y, lam, terms=toms_terms, eps=1e-15, tiny=1e-30, maxit=3000):
    b0, T = terms(a, b, x, y, lam)
    f = tiny; C = f; D = 0.0
    D = b0 + 1.0 * D; D = 1.0 / D; C = b0 + 1.0 / C; f = f * (C * D)
    for k, (al, be) in enumerate(T):
        D = be + al * D
        D = 1.0 / D
        C = be + al / C
        de = C * D; f = f * de
        if abs(de - 1.0) <= eps or k >= maxit: return f, k + 1

def fma(a, b, c):
    """Correctly rounded a*b + c on doubles."""
    return float(Fr(a) * Fr(b) + Fr(c))

def toms_terms_opt(fx=0, fb=0, ext=False, w_ord=0, be_ord=0):
    """TOMS terms with options: fx fuses c + n*yp1; fb fuses e*X into beta's sum; ext evaluates each statement in
    x87 extended (numpy longdouble) and stores a double; w_ord 1 = n*x*(b-n) (R), be_ord 1 = w/s + (n + e*X) (R)."""
    import numpy as np
    LD = np.longdouble if ext else float
    def mk(a, b, x, y, lam):
        c = float(LD(lam) + LD(1)); c0 = float(LD(b) / LD(a)); c1 = float(LD(1) / LD(a) + LD(1)); yp1 = float(LD(y) + LD(1))
        st = dict(n=0.0, p=1.0, s=a + 1.0)
        def gen():
            while True:
                n = st['n'] + 1.0; p, s = st['p'], st['s']
                N, A_, B_, X_, P_, S_ = LD(n), LD(a), LD(b), LD(x), LD(p), LD(s)
                w = float(N * X_ * (B_ - N)) if w_ord else float(N * (B_ - N) * X_)
                t = float(N / A_); e = float(A_ / S_)
                E = LD(e); W = LD(w)
                alpha = float(P_ * (P_ + LD(c0)) * E * E * (W * X_))
                T = LD(t)
                e2 = float((T + LD(1)) / (LD(c1) + T + T))
                Xs = fma(n, yp1, c) if fx else float(LD(c) + N * LD(yp1))
                if fb:
                    beta = fma(e2, Xs, float(W / S_ + N) if be_ord else float(N + W / S_))
                else:
                    beta = float(W / S_ + (N + LD(e2) * LD(Xs))) if be_ord else float(N + W / S_ + LD(e2) * LD(Xs))
                st['n'] = n; st['p'] = float(T + LD(1)); st['s'] = s + 2.0
                yield alpha, beta
        return float(LD(c) / LD(c1)), gen()
    return mk

B0_FORMS = {
    'c/c1': lambda a, c: c / (1.0 / a + 1.0),
    'c/((a+1)/a)': lambda a, c: c / ((a + 1.0) / a),
    'c*a/(a+1)': lambda a, c: c * a / (a + 1.0),
    'c*(a/(a+1))': lambda a, c: c * (a / (a + 1.0)),
    'a*c/(a+1)': lambda a, c: a * c / (a + 1.0),
    '(a+a*lam)/(a+1)': None,
    'c/(1+1/a)': lambda a, c: c / (1.0 + 1.0 / a),
    'c*(1/c1)': lambda a, c: c * (1.0 / (1.0 / a + 1.0)),
}
E2_FORMS = {
    'toms (t+1)/(c1+t+t)': None,
    '(a+n)/(a+1+2n)': lambda a, n: (a + n) / (a + 1.0 + 2.0 * n),
    '(a+n)/(a+2n+1)': lambda a, n: (a + n) / (a + 2.0 * n + 1.0),
    '(a+n)/(s+2)': 's',
    '(n+a)/(2n+a+1)': lambda a, n: (n + a) / (2.0 * n + a + 1.0),
}

def hybrid_terms(b0f, e2f):
    def mk(a, b, x, y, lam):
        c = lam + 1.0; c0 = b / a; c1 = 1.0 / a + 1.0; yp1 = y + 1.0
        b0 = (a + a * lam) / (a + 1.0) if B0_FORMS[b0f] is None else B0_FORMS[b0f](a, c)
        def gen():
            n = 0.0; p = 1.0; s = a + 1.0
            while True:
                n += 1.0; w = n * (b - n) * x; t = n / a; e = a / s
                alpha = p * (p + c0) * e * e * (w * x)
                f2 = E2_FORMS[e2f]
                if f2 is None: e = (t + 1.0) / (c1 + t + t)
                elif f2 == 's': e = (a + n) / (s + 2.0)
                else: e = f2(a, n)
                beta = n + w / s + e * (c + n * yp1)
                p = t + 1.0; s += 2.0
                yield alpha, beta
        return b0, gen()
    return mk

# Explicit term forms. u = a+n-1, v = a+b+n-1, s = a+2n-1, m = n*(b-n), w = m*x, X = c + n*yp1, e2 = (a+n)/(a+2n+1).
ALPHA_FORMS = {
    'toms': None,
    'u*v*m*x*x/(s*s)': lambda u, v, s, m, w, x, a, b, n: u * v * m * x * x / (s * s),
    'u*v/(s*s)*m*x*x': lambda u, v, s, m, w, x, a, b, n: u * v / (s * s) * m * x * x,
    '(u/s)*(v/s)*(w*x)': lambda u, v, s, m, w, x, a, b, n: (u / s) * (v / s) * (w * x),
    'u/s*v/s*w*x': lambda u, v, s, m, w, x, a, b, n: u / s * v / s * w * x,
    'u*v*(w*x)/(s*s)': lambda u, v, s, m, w, x, a, b, n: u * v * (w * x) / (s * s),
    '(u*v)/(s*s)*(w*x)': lambda u, v, s, m, w, x, a, b, n: (u * v) / (s * s) * (w * x),
    'u*v/(s*s)*(w*x)': lambda u, v, s, m, w, x, a, b, n: u * v / (s * s) * (w * x),
    'u*v*w*x/(s*s)': lambda u, v, s, m, w, x, a, b, n: u * v * w * x / (s * s),
    'u*v*w*x/s/s': lambda u, v, s, m, w, x, a, b, n: u * v * w * x / s / s,
    'p(p+c0)e e(wx) p=u/a': lambda u, v, s, m, w, x, a, b, n: (u / a) * ((u / a) + b / a) * (a / s) * (a / s) * (w * x),
    'p(p+c0)ee(wx) p=u/a,pc0=v/a': lambda u, v, s, m, w, x, a, b, n: (u / a) * (v / a) * (a / s) * (a / s) * (w * x),
    'u*v*x*x*m/(s*s)': lambda u, v, s, m, w, x, a, b, n: u * v * x * x * m / (s * s),
    'n*u*v*(b-n)*x*x/(s*s)': lambda u, v, s, m, w, x, a, b, n: n * u * v * (b - n) * x * x / (s * s),
    '(u*v*w*x)/(s*s)': lambda u, v, s, m, w, x, a, b, n: (u * v * w * x) / (s * s),
    'w*x*u*v/(s*s)': lambda u, v, s, m, w, x, a, b, n: w * x * u * v / (s * s),
    'u*v/s/s*w*x': lambda u, v, s, m, w, x, a, b, n: u * v / s / s * w * x,
    'v/s/s*u*m*x*x': lambda u, v, s, m, w, x, a, b, n: v / s / s * u * m * x * x,
}
BETA_FORMS = {
    'n + w/s + e2*X': lambda n, w, s, e2, X, a, lam, y, m, x: n + w / s + e2 * X,
    'w/s + (n + e2*X)': lambda n, w, s, e2, X, a, lam, y, m, x: w / s + (n + e2 * X),
    'n + (w/s + e2*X)': lambda n, w, s, e2, X, a, lam, y, m, x: n + (w / s + e2 * X),
    'n + w/s + (a+n)*X/(a+2n+1)': lambda n, w, s, e2, X, a, lam, y, m, x: n + w / s + (a + n) * X / (a + 2 * n + 1),
    'n + m*x/s + e2*X': lambda n, w, s, e2, X, a, lam, y, m, x: n + m * x / s + e2 * X,
    'n + w/s + e2*(1+lam+n*(1+y))': lambda n, w, s, e2, X, a, lam, y, m, x: n + w / s + e2 * (1 + lam + n * (1 + y)),
    'n + w/s + e2*(lam+1+n*(y+1))': lambda n, w, s, e2, X, a, lam, y, m, x: n + w / s + e2 * (lam + 1 + n * (y + 1)),
    'n + w/s + e2*(c+n+n*y)': lambda n, w, s, e2, X, a, lam, y, m, x: n + w / s + e2 * ((lam + 1) + n + n * y),
    'R: n + w/s + e2*X': lambda n, w, s, e2, X, a, lam, y, m, x: n + w / s + e2 * X,   # w = n*x*(b-n) (R's order)
}
def explicit_terms(alf, bef, b0f='c*(a/(a+1))'):
    def mk(a, b, x, y, lam):
        c = lam + 1.0; c0 = b / a; yp1 = y + 1.0
        b0 = B0_FORMS[b0f](a, c)
        def gen():
            n = 0.0; p = 1.0; s_t = a + 1.0
            while True:
                n += 1.0; u = a + n - 1.0; v = a + b + n - 1.0; s = a + 2.0 * n - 1.0; m = n * (b - n); w = m * x
                if ALPHA_FORMS[alf] is None:
                    e = a / s_t; alpha = p * (p + c0) * e * e * (w * x); p = n / a + 1.0; s_t += 2.0
                else:
                    alpha = ALPHA_FORMS[alf](u, v, s, m, w, x, a, b, n)
                e2 = (a + n) / (a + 2.0 * n + 1.0); X = c + n * yp1
                wb = n * x * (b - n) if bef.startswith('R:') else w
                beta = BETA_FORMS[bef](n, wb, s, e2, X, a, lam, y, m, x)
                yield alpha, beta
        return b0, gen()
    return mk

def lentz2(a, b, x, y, lam, terms, eps=2.220446049250313e-15, tiny=1e-30, du=0, cu=0, de_=0, stop=0, maxit=3000):
    """Lentz with options. du: D = 1/(be + al*D) | 1/fma(al, D, be). cu: C = be + al/C | fma? no: be + al*(1/C).
    de_: f *= C*D | f = f*C*D (left) | f *= (D*C). stop: |de-1| <= eps | < eps | |f - f0| <= eps*|f|."""
    b0, T = terms(a, b, x, y, lam)
    f = tiny; C = f; D = 0.0
    D = b0 + 1.0 * D; D = 1.0 / D; C = b0 + 1.0 / C
    f = f * (C * D) if de_ != 1 else f * C * D
    for k, (al, be) in enumerate(T):
        D = 1.0 / (fma(al, D, be) if du else be + al * D)
        C = be + al * (1.0 / C) if cu else be + al / C
        f0 = f
        if de_ == 1: de = C * D; f = f * C * D
        elif de_ == 2: de = D * C; f = f * de
        else: de = C * D; f = f * de
        if stop == 0 and abs(de - 1.0) <= eps: break
        if stop == 1 and abs(de - 1.0) < eps: break
        if stop == 2 and abs(f - f0) <= eps * abs(f): break
        if k >= maxit: break
    return f, k + 1

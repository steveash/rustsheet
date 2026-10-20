"""Coefficient-free structure fingerprint (T-1941 W5; W2's method, re-implemented for cross-checking).
For each point, take the target quotient A*: the mean of the doubles A that reproduce Excel's y through the kernel's
outer ops. Fit the function by weighted linear LS of N(x) - A* D(x) in a Chebyshev basis on the data interval
(normalised by D's T0 coefficient, which leaves N/D unchanged; iterative refinement in long double). The residual
rms in ulp(A) is the rounding noise of the true op structure. Calibrate by simulating each candidate structure with
the fitted function: to_monomial() converts to raw-x monomials with a given D0 at 50 digits and rounds to doubles."""
import numpy as np, mpmath as mp
L = np.longdouble
def st(v): return np.asarray(v, dtype=L).astype(np.float64)
def nbr(a, k):
    r = a.copy()
    for _ in range(abs(k)): r = np.nextafter(r, np.inf if k > 0 else -np.inf)
    return r
def target(y, outer, a0, K=4):
    """A* = mean of the doubles A within +-K ulps of a0 with outer(A) == y (nan if none)."""
    s = np.zeros(len(y), dtype=L); n = np.zeros(len(y))
    for k in range(-K, K+1):
        a = nbr(a0, k); h = outer(a) == y; s[h] += L(a[h]); n[h] += 1
    out = np.full(len(y), np.nan, dtype=L); m = n > 0; out[m] = s[m]/L(n[m]); return out
class Cheb:
    def __init__(self, lo, hi): self.lo, self.hi = lo, hi
    def s(self, x): return (L(2)*L(x)-L(self.lo)-L(self.hi))/(L(self.hi)-L(self.lo))
    def basis(self, x, n):
        s = self.s(x); T = [np.ones_like(s), s]
        for _ in range(2, n+1): T.append(L(2)*s*T[-1]-T[-2])
        return T[:n+1]
def fit(x, A, nN, nD, lo, hi, it=10):
    """LS fit of N - A D in Chebyshev coefficients (b_0 = 1). Returns (a, b, residual in ulp(A))."""
    ch = Cheb(lo, hi); TN = ch.basis(x, nN); TD = ch.basis(x, nD)
    Al = L(A); u = L(np.spacing(np.abs(A.astype(float))))
    a = np.zeros(nN+1, dtype=L); b = np.zeros(nD+1, dtype=L); b[0] = 1
    best = (np.inf, a.copy(), b.copy())
    for _ in range(it):
        Nv = sum(a[k]*TN[k] for k in range(nN+1)); Dv = sum(b[k]*TD[k] for k in range(nD+1))
        rr = ((Nv/Dv-Al)/u).astype(float); q = float(np.sqrt(np.mean(rr*rr)))
        if q < best[0]: best = (q, a.copy(), b.copy())
        w = L(1)/(np.abs(Dv)*u)
        r = ((Al*Dv-Nv)*w).astype(float)
        M = np.array([(TN[k]*w).astype(float) for k in range(nN+1)] + [(-TD[k]*Al*w).astype(float) for k in range(1, nD+1)]).T
        sc = np.sqrt((M*M).sum(0)); sc[sc == 0] = 1
        d, *_ = np.linalg.lstsq(M/sc, r, rcond=1e-30); d = d/sc
        a += d[:nN+1].astype(L); b[1:] += d[nN+1:].astype(L)
    Nv = sum(a[k]*TN[k] for k in range(nN+1)); Dv = sum(b[k]*TD[k] for k in range(nD+1))
    rr = ((Nv/Dv-Al)/u).astype(float)
    if float(np.sqrt(np.mean(rr*rr))) < best[0]: return a, b, rr
    a, b = best[1], best[2]
    Nv = sum(a[k]*TN[k] for k in range(nN+1)); Dv = sum(b[k]*TD[k] for k in range(nD+1))
    return a, b, ((Nv/Dv-Al)/u).astype(float)
def to_monomial(a, b, lo, hi, D0):
    """Chebyshev (a, b) on [lo, hi] -> raw-x monomial (N, D) with D(0) = D0, rounded to doubles."""
    mp.mp.dps = 50
    def conv(c):
        n = len(c)-1; T = [[mp.mpf(1)], None]
        # s = alpha x + beta
        al = mp.mpf(2)/(mp.mpf(hi)-mp.mpf(lo)); be = -(mp.mpf(lo)+mp.mpf(hi))/(mp.mpf(hi)-mp.mpf(lo))
        T[1] = [be, al]
        for _ in range(2, n+1):
            p, q = T[-1], T[-2]; r = [mp.mpf(0)]*(len(p)+1)
            for i, v in enumerate(p): r[i] += 2*be*v; r[i+1] += 2*al*v
            for i, v in enumerate(q): r[i] -= v
            T.append(r)
        out = [mp.mpf(0)]*(n+1)
        for k in range(n+1):
            for i, v in enumerate(T[k]): out[i] += mp.mpf(float(c[k])) + 0 if False else 0
        # long double coefficients: use their exact value via two doubles
        out = [mp.mpf(0)]*(n+1)
        for k in range(n+1):
            ck = mp.mpf(float(c[k])) + mp.mpf(float(c[k]-L(float(c[k]))))
            for i, v in enumerate(T[k]): out[i] += ck*v
        return out
    Nm, Dm = conv(a), conv(b); sc = mp.mpf(D0)/Dm[0]
    return np.array([float(v*sc) for v in Nm]), np.array([float(v*sc) for v in Dm])
def stats(r):
    r = r[np.isfinite(r)]; return float(np.sqrt(np.mean(r*r))), float(np.median(np.abs(r)))

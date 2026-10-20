"""Function-free structure fingerprint for t*N/D kernels (T-1941, W2). Target A* from Excel y (mean of doubles A with
fl(t*A)==y); LS fit of N - A* D in a Chebyshev correction basis on a long-double function; residual rms/median in ulp(A).
CALIBRATE WITH PLANTED TRUTHS FROM FIXED REAL COEFFICIENTS (sim(x, x0, P, Q, unN, unD) then fit): coefficient bits alone
move the level by ~+-0.03. Stage i (high-first): P_i = s_{i-1}*x, S_i = P_i + c[i]; ops not in the unstored sets are fl53(fl64)."""
import numpy as np
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
def chain(c, X, un):
    s = LD(c[0]) + 0 * X
    for i in range(1, len(c)):
        P = s * X
        if ('P', i) not in un: P = sd(P)
        v = P + LD(c[i]); s = v if ('S', i) in un else sd(v)
    return s
def sim(x, x0, p, q, unN=frozenset(), unD=frozenset()):
    X = np.asarray(x).astype(LD); return sd(sd(X - LD(x0)) * sd(chain(p, X, unN) / chain(q, X, unD))).astype(np.float64)
def astar(x, b, x0):
    t = x - x0; tz = np.where(t == 0, 1.0, t); a0 = b / tz; acc = np.zeros(len(x)); ns = np.zeros(len(x))
    for k in range(-4, 5):
        ak = a0.copy()
        for _ in range(abs(k)): ak = np.nextafter(ak, np.inf if k > 0 else -np.inf)
        ok = ((LD(1) * t * ak).astype(np.float64) == b); acc += np.where(ok, ak, 0); ns += ok
    g = (ns > 0) & (t != 0); return g, acc / np.where(ns > 0, ns, 1)
def ev(c, X):
    X = X.astype(LD); s = LD(c[0]) + 0 * X
    for v in c[1:]: s = s * X + v
    return s
def _cheb(u, k):
    T = [LD(1) + 0 * u, u]
    while len(T) < k: T.append(2 * u * T[-1] - T[-2])
    return T[:k]
def _tomono(a, lo, hi):
    """Chebyshev coefficients (u=(2x-lo-hi)/(hi-lo)) -> monomial coefficients in x, high-first (mpmath)"""
    from mpmath import mp, mpf
    mp.dps = 60; n = len(a)
    s = mpf(2) / (mpf(hi) - mpf(lo)); o = -(mpf(lo) + mpf(hi)) / (mpf(hi) - mpf(lo))
    polys = [[mpf(1)], [o, s]]
    while len(polys) < n:
        P1 = polys[-1]; P0 = polys[-2]; r = [mpf(0)] * (len(P1) + 1)
        for i, v in enumerate(P1): r[i] += 2 * o * v; r[i + 1] += 2 * s * v
        for i, v in enumerate(P0): r[i] -= v
        polys.append(r)
    out = [mpf(0)] * n
    for k in range(n):
        for i, v in enumerate(polys[k]): out[i] += mpf(str(a[k])) * v
    return [LD(str(v)) for v in out[::-1]]
def fit(x, b, x0, p, q, qfix=None, its=4, verbose=False):
    """returns (p, q) long-double monomials (high-first), residual rms, median|r| (ulp A); |r|>20 dropped (data glitches)"""
    g, A = astar(x, b, x0); xg = x[g]; A = A[g]; n, m = len(p), len(q)
    lo, hi = float(xg.min()), float(xg.max()); u = (2 * xg.astype(LD) - LD(lo) - LD(hi)) / (LD(hi) - LD(lo))
    TN = _cheb(u, n); TD = _cheb(u, m); aN = np.zeros(n); aD = np.zeros(m)
    base_N = ev([LD(v) for v in p], xg); base_D = ev([LD(v) for v in q], xg)
    def cur():
        N = base_N + sum(LD(c) * T for c, T in zip(aN, TN)); D = base_D + sum(LD(c) * T for c, T in zip(aD, TD)); return N, D
    for it in range(its):
        N, D = cur(); r = N - LD(1) * A * D; w = 1.0 / (np.spacing(np.abs(A)) * np.abs(np.asarray(D, np.float64)))
        Af = np.asarray(A, np.float64)
        M = np.stack([np.asarray(T, np.float64) for T in TN] + [-Af * np.asarray(T, np.float64) for T in TD[1:]], 1) * w[:, None]
        s_ = np.linalg.norm(M, axis=0); M /= s_; rr = np.asarray(r, np.float64) * w
        keep = np.abs(rr) < 20 if it > 0 else np.ones(len(rr), bool)
        sol, *_ = np.linalg.lstsq(M[keep], -rr[keep], rcond=None); sol /= s_
        if verbose: print('   it %d rms %.4f' % (it, np.sqrt(np.mean(rr[np.abs(rr) < 20] ** 2))))
        aN += sol[:n]; aD[1:] += sol[n:]
    N, D = cur(); r = np.asarray(N - LD(1) * A * D, np.float64) / (np.spacing(np.abs(A)) * np.abs(np.asarray(D, np.float64)))
    r = r[np.abs(r) < 20]
    pN = _tomono(aN, lo, hi); pD = _tomono(aD, lo, hi)
    return [LD(v) + d for v, d in zip(p, pN)], [LD(v) + d for v, d in zip(q, pD)], float(np.sqrt(np.mean(r ** 2))), float(np.median(np.abs(r)))
def ops_from(i0, i1, kinds='PS'):
    return frozenset((k, i) for i in range(i0, i1 + 1) for k in kinds)

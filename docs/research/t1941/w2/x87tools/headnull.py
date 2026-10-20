"""Function-preserving head moves: dp0 (and others) fixed, rest of the head LS-compensated on [2,3].
control used: cH = (moved(p,4,{0:1e-16,1:3e-17}), moved(q,3,{0:2e-19}))"""
import sys, os, numpy as np, math
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', '..', 'review3'))
from spec import load, p as P0, q as Q0
from mpmath import mp, mpf, matrix, lu_solve
mp.dps = 50
xs = [mpf(2) + mpf(i) / 40 for i in range(41)]
def compensate(c, deg, fixed):
    free = [i for i in range(deg + 1) if i not in fixed]
    A = [[x ** (deg - i) for i in free] for x in xs]; bvec = [-sum(fixed[i] * x ** (deg - i) for i in fixed) for x in xs]
    n = len(free); M = matrix(n, n); v = matrix(n, 1)
    for r in range(len(xs)):
        for i in range(n):
            v[i] += A[r][i] * bvec[r]
            for j in range(n): M[i, j] += A[r][i] * A[r][j]
    sol = lu_solve(M, v); delta = dict(fixed)
    for k, i in enumerate(free): delta[i] = sol[k]
    resid = max(abs(sum(delta[i] * x ** (deg - i) for i in range(deg + 1))) for x in xs)
    return delta, resid
def moved(base, deg, fixed):
    """base: coefficient list (highest first); deg: head degree (4 for N, 3 for D); fixed: {index: mpf delta}"""
    delta, resid = compensate([mpf(v) for v in base[:deg + 1]], deg, fixed)
    new = list(base)
    for i in range(deg + 1): new[i] = float(mpf(base[i]) + delta[i])
    return new, float(resid)

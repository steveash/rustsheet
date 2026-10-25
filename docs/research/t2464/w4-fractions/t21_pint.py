"""Is bup's internal density the rounded worksheet value or the extended EXP result?  On cells where w1's M1 model
reproduces Excel's worksheet density exactly, compare cdf matches using P_ws (double) and M1's extended P."""
import json, collections, mpmath
from common4 import *
import tomsx
from tomsx import mpf, w1
out = json.load(open('probe/w4p1.out.json'))['sheets']; meta = json.load(open('probe/w4p1.meta.json'))
data = {(r['x'], r['a'], r['b']): r for r in load(lambda d: d['branch'] == 'direct' and d['src'] == 'w2')}
tomsx.prefetch('LN', [1 - m[2] for m in meta] + [m[2] for m in meta])
R64, D = w1.R64, w1.D
cnt = collections.Counter(); per = collections.defaultdict(collections.Counter)
for sh, i, xx, a, b, b0 in meta:
    if sh == 'S2': continue
    c = out[sh]; P = c[f'B{i}']['v']; Q = c[f'C{i}']['v']; y0 = c[f'D{i}']['v']; E = data[(xx, a, b)]['cdf']
    n = int(b)
    if b - n == 0: n -= 1
    A, B, X = b0, a, y0
    L = mpf(tomsx.lnB_small(A, B))
    lnx = mpf(tomsx.XL('LN', X)); ly = mpf(tomsx.XL('LN', 1.0 - X))
    tx = mpf(D(mpf(D(mpf(A) - 1)) * lnx)); ty = mpf(D(mpf(D(mpf(B) - 1)) * ly))
    if A == 1.0:   # w1: b = 1 / a = 1 special handling: here first shape 1 -> tx = 0
        pass
    Ex = R64(tx + R64(ty - L))
    Pd = w1.exp_v6(Ex); Pe = w1.exp_v6_ext(Ex)
    if Pd != P: cnt['m1 miss'] += 1; continue
    apb, ap1 = b0 + a, b0 + 1.0; d = 1.0; w = 1.0
    for l in range(n - 1): d = d * ((apb + l) / (ap1 + l) * y0); w = w + d
    m_d = (P * (y0 * xx)) / b0 * w + Q
    s1 = LD(y0 * xx)
    pe = LD(mpmath.nstr(Pe, 25))
    m_e1 = float(LD(float(pe * s1)) / LD(b0)) * w + Q        # ext P, product stored
    m_e2 = float(float(pe * s1) / b0 * w) + Q
    cnt['m1 hit'] += 1
    for k, m in (('P double', m_d), ('P ext', m_e1)):
        cnt[k] += m == E; per[k][sh] += m == E
print(dict(cnt)); print({k: dict(v) for k, v in per.items()})

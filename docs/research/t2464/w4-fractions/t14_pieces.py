"""With Excel's own pieces from probe w4p1 (density at y0 = 1-x of Beta(b0', a), and I_x(a, b0')), test
cdf = fl( fl(fl(P*fl(y0*x))/b0' * W(y0)) + Q )  (TOMS 708 bratio L140: bup + bpser) and variants."""
import json, collections, itertools
from common4 import *
out = json.load(open('probe/w4p1.out.json'))['sheets']
meta = json.load(open('probe/w4p1.meta.json'))
data = {(r['x'], r['a'], r['b']): r for r in load(lambda d: d['branch'] == 'direct' and d['src'] == 'w2')}
def W_(X, A, B, n, mode):
    apb, ap1 = A + B, A + 1.0; d = 1.0; w = 1.0
    for l in range(n - 1):
        if mode == 'sse': d = d * ((apb + l) / (ap1 + l) * X); w = w + d
        elif mode == 'sse2': d = d * (apb + l) / (ap1 + l) * X; w = w + d
        else:
            d = x('*', d, x('*', x('/', apb + l, ap1 + l), X)); w = x('+', w, d)
    return w
cnt = collections.Counter(); per = collections.defaultdict(collections.Counter); nrow = collections.Counter(); chk = collections.Counter()
for sh, i, xx, a, b, b0 in meta:
    c = out[sh]; P = c[f'B{i}']['v']; Q = c[f'C{i}']['v']; y0 = c[f'D{i}']['v']
    r = data[(xx, a, b)]; E = r['cdf']
    chk['y0==fl(1-x)'] += y0 == 1 - xx
    n = int(b); 
    if b - n == 0: n -= 1
    nrow[sh] += 1
    for wm, order, dv in itertools.product(('sse', 'sse2', 'x87'), ('p(xy)', 'pxy', 'p(yx)'), ('ret/a*w', 'ret*w/a', '(p/a)...')):
        W = W_(y0, b0, a, n, wm)
        if order == 'p(xy)': brc = P * (y0 * xx)
        elif order == 'pxy': brc = (P * y0) * xx
        else: brc = P * (xx * y0)
        if dv == 'ret/a*w': up = (brc / b0) * W
        elif dv == 'ret*w/a': up = (brc * W) / b0
        else: up = ((P / b0) * (y0 * xx)) * W
        m = up + Q
        k = (wm, order, dv); cnt[k] += m == E; per[k][sh] += m == E
print(dict(chk), dict(nrow))
for k, v in cnt.most_common(8): print(k, v, dict(per[k]))

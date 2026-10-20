import sys, os, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from k3lib import *
LD = np.longdouble
x, b, _ = rows(sub=int(os.environ.get('SUB', 2000000)), seed=7); xh, bh, _ = rows(sub=500000, seed=8)
p0, q0 = ldset('docs/research/t1941/w5/p4/ss_fc.txt')
t = x - 1.0; tz = np.where(t == 0, 1.0, t); a0 = b / tz; acc = np.zeros(len(x)); ns = np.zeros(len(x))
for k in range(-4, 5):
    ak = a0.copy()
    for _ in range(abs(k)): ak = np.nextafter(ak, np.inf if k > 0 else -np.inf)
    ok = ((LD(1) * t * ak).astype(np.float64) == b); acc += np.where(ok, ak, 0); ns += ok
good = (ns > 0) & (t != 0); Astar = acc[good] / ns[good]; xg = x[good]
def ev(c, xx):
    X = xx.astype(LD); s = LD(c[-1]) + 0 * X
    for v in c[-2::-1]: s = s * X + LD(v)
    return s
p, q = list(p0), list(q0)
for it in range(3):
    Nv = ev(p, xg); Dv = ev(q, xg); r = Nv - LD(1) * Astar * Dv
    w = 1.0 / (np.spacing(np.abs(Astar)) * np.abs(np.asarray(Dv, np.float64)))
    M = np.stack([xg ** i for i in range(10)] + [-Astar * xg ** i for i in range(1, 10)], 1) * w[:, None]; s_ = np.linalg.norm(M, axis=0); M /= s_
    rr = np.asarray(r, np.float64) * w; sol, *_ = np.linalg.lstsq(M, -rr, rcond=None); sol /= s_
    p = [float(LD(v) + LD(d)) for v, d in zip(p, sol[:10])]; q = [q[0]] + [float(LD(v) + LD(d)) for v, d in zip(q[1:], sol[10:])]
    print('it %d: resid rms %.3f ulp(A); exact stored %.4f holdout %.4f' % (it, np.sqrt(np.mean(rr ** 2)), score(x, b, p, q), score(xh, bh, p, q)), flush=True)

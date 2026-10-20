"""Function correction by CVP (T-1941 W5 part 5): integer moves of N3..N9, D3..D9 (ulps) that take START's function
to the real-data LS function fit (w5/fp: Chebyshev (9,9) LS of N - A* D, residual ~1 ulp(A)), i.e. the closest
vector to [0 | lam t] in the lattice [I | lam J], t = (A_fit - A_start)/ulp(A) on a grid.
usage: k3cvp.py START LAM OUT   env: NSUB (fit points, 300000)"""
import numpy as np, mpmath as mp, math, sys, os
from fpylll import IntegerMatrix, BKZ, CVP, LLL
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..', 'fp'))
from hypolib import ld
from fp import L, st, target, fit, Cheb
mp.mp.dps = 40
N0, D0 = ld(sys.argv[1]); lam = float(sys.argv[2])
z = np.load('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows6.npz')
i = np.random.default_rng(31).choice(len(z['x']), int(os.environ.get('NSUB', 300000)), replace=False); x, y = z['x'][i], z['b'][i]
m = np.abs(x-1) > 1e-2; x, y = x[m], y[m]; tt = st(L(x)-L(1.0))
A = target(y, lambda a: st(L(tt)*L(a)), st(L(y)/L(tt)))
a, b, r = fit(x, A, 9, 9, 0.7, 1.5); print('LS fit residual rms %.3f ulp(A)' % float(np.sqrt(np.mean(r*r))))
ch = Cheb(0.7, 1.5)
def Afit(xx):
    TN = ch.basis(np.array(xx), 9); return sum(a[k]*TN[k] for k in range(10))/sum(b[k]*TN[k] for k in range(10))
idx = [('N', j) for j in range(3, 10)] + [('D', j) for j in range(3, 10)]
ul = np.array([math.ulp(abs((N0 if c == 'N' else D0)[j])) for c, j in idx])
grid = np.linspace(0.7, 1.5, 81); J = []; tv = []
af = Afit(grid)
for g, afv in zip(grid, af):
    t_ = mp.mpf(g); Nv = sum(mp.mpf(N0[j])*t_**j for j in range(10)); Dv = sum(mp.mpf(D0[j])*t_**j for j in range(10)); Av = Nv/Dv
    u = mp.mpf(2)**(mp.floor(mp.log(abs(Av), 2))-52)
    J.append([float(t_**j/Dv/u) if c == 'N' else float(-Av*t_**j/Dv/u) for c, j in idx])
    tv.append(float((mp.mpf(float(afv))+mp.mpf(float(afv-L(float(afv))))-Av)/u))
J = np.array(J)*ul; tv = np.array(tv)
print('function gap start -> LS fit on grid: max %.3f, rms %.3f ulp(A)' % (np.abs(tv).max(), np.sqrt(np.mean(tv**2))))
n, mm, sc = len(idx), len(grid), 10**4
B = IntegerMatrix(n, n+mm)
for j in range(n):
    B[j, j] = sc
    for k in range(mm): B[j, n+k] = int(round(lam*J[k, j]*sc))
LLL.reduction(B)
tgt = [0]*n + [int(round(lam*tv[k]*sc)) for k in range(mm)]
v = CVP.closest_vector(B, tgt)
mv = np.array([int(round(v[j]/sc)) for j in range(n)])
res_ = J@mv-tv
print('move: ' + ' '.join('%s%d:%+d' % (c, j, d) for (c, j), d in zip(idx, mv) if d))
print('after move: function gap max %.3f rms %.3f ulp(A)' % (np.abs(res_).max(), np.sqrt(np.mean(res_**2))))
N1, D1 = N0.copy(), D0.copy()
for (c, j), d in zip(idx, mv):
    C = N1 if c == 'N' else D1; C[j] += d*np.spacing(abs(C[j]))
open(sys.argv[3], 'w').write(' '.join(v_.hex() for v_ in N1)+'\n'+' '.join(v_.hex() for v_ in D1)+'\n')

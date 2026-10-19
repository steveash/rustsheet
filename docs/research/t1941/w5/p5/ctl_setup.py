"""Peeling control files: truth = synthetic function-preserving move of ss_fc; base = truth with c_J moved by K0 ulps
and the lower coefficients refitted to restore the function (so everything above J is exact in the base)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from synth import make_truth
from meter import L
CH, J, K0 = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
N0, D0 = ld(os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))
Nt, Dt, mv, fc = make_truth(N0, D0, 1, 1e5)
def wr(f, N, D): open(os.path.join(HERE, f), 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')
wr('ctl_truth.txt', Nt, Dt)
N1 = Nt.copy(); D1 = Dt.copy(); C1 = N1 if CH == 'N' else D1; C1[J] += K0*np.spacing(abs(C1[J]))
free = [(CH, i) for i in range(J)] + [(o, i) for o in ('N', 'D') if o != CH for i in range(1, J)]
xs = np.linspace(0.7, 1.5, 120).astype(L); tgt = np.polyval(Nt.astype(L)[::-1], xs)/np.polyval(Dt.astype(L)[::-1], xs)
for it in range(3):
    Nv = np.polyval(N1.astype(L)[::-1], xs); Dv = np.polyval(D1.astype(L)[::-1], xs)
    M = np.array([(np.spacing(abs((N1 if c == 'N' else D1)[i]))*xs**i/(Nv if c == 'N' else -Dv)).astype(float) for c, i in free]).T
    sol, *_ = np.linalg.lstsq(M/1e-18, ((tgt*Dv/Nv)-1).astype(float)/1e-18, rcond=None)
    for (c, i), v in zip(free, sol):
        CC = N1 if c == 'N' else D1; CC[i] += round(v)*np.spacing(abs(CC[i]))
f = np.abs((np.polyval(N1.astype(L)[::-1], xs)/np.polyval(D1.astype(L)[::-1], xs))/tgt-1).max()
wr('ctl_base.txt', N1, D1); print('base: %s%d moved %+d ulps, lower refit; function diff %.1e' % (CH, J, K0, float(f)))

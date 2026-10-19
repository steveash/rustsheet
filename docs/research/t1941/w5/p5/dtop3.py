"""D top, last step: (D7, D8, D9) on the J0=6 objective (sites through S6, P5 excluded so D6 whole-u shifts are
free), which separates the D top far better than the J0=7 one (control: truth 712 vs base 55). Coarse grid, then fine.
usage: dtop3.py control|real START OUT   env: BASE, TRUTHFILE, R7, R8, S8, R9, S9 (D7/D8/D9 ranges and steps)"""
import numpy as np, sys, os, time
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import Data, ld
mode, start, out = sys.argv[1:4]
N0, D0 = ld(os.environ.get('BASE', os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))); Ns, Ds = ld(start)
d = Data(N0, D0, mode, os.environ.get('TRUTHFILE'))
u = [float(np.spacing(abs(Ds[i]))) for i in range(10)]
R7, R8, S8, R9, S9 = (int(float(os.environ.get(k, v))) for k, v in (('R7', 32), ('R8', 4), ('S8', 2), ('R9', 2**19), ('S9', 8192)))
def grid(c, k7, k8, k9):
    return np.array([[c[0], c[1]+a*u[7], c[2]+b*u[8], c[3]+e*u[9]] for a in k7 for b in k8 for e in k9])
def run(c, k7, k8, k9, tag):
    t0 = time.time(); H = grid(c, k7, k8, k9); v = d.obj('D', 6, H, 0); o = np.argsort(v)[::-1]; mu, sd = np.median(v), v.std()
    st = lambda h: 'dD7 %+d dD8 %+d dD9 %+d' % tuple(round((h[k]-Ds[6+k])/u[6+k]) for k in (1, 2, 3))
    print('%s: %d hyps %.0fs; top: %s' % (tag, len(H), time.time()-t0, ' | '.join('%s %.0f (Z %.1f)' % (st(H[i]), v[i], (v[i]-mu)/sd) for i in o[:4])), flush=True)
    return H[o[0]], v[o[0]], v[o[1]] if len(o) > 1 else np.nan
c0 = Ds[6:].copy()
b, vb, _ = run(c0, range(-R7, R7+1), range(-R8, R8+1, S8), range(-R9, R9+1, S9), 'coarse')
b, vb, v2 = run(b, range(-2, 3), range(-2, 3), range(-S9, S9+1, S9//32), 'fine')
D1 = Ds.copy(); D1[6:] = b
open(out, 'w').write(' '.join(x_.hex() for x_ in Ns)+'\n'+' '.join(x_.hex() for x_ in D1)+'\n')
print('best %.0f (next %.0f)' % (vb, v2))
if mode == 'control':
    Dt = ld(os.environ['TRUTHFILE'])[1]
    print('found - truth (ulps): D7 %+d D8 %+d D9 %+d; truth objective %.0f' % (*[round((b[k]-Dt[6+k])/u[6+k]) for k in (1, 2, 3)],
          d.obj('D', 6, np.array([[b[0], *Dt[7:]]]), 0)[0]))

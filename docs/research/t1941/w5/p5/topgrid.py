"""Local exact-objective grid for the N-chain top (N7, N8, N9) around a start set (T-1941 W5 part 5).
usage: topgrid.py control|real START OUT   env: BASE (reference for residual/weights), TRUTHFILE (control)"""
import numpy as np, sys, os, time
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import Data, ld
mode, start, out = sys.argv[1], sys.argv[2], sys.argv[3]
N0, D0 = ld(os.environ.get('BASE', os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))); Ns, Ds = ld(start)
d = Data(N0, D0, mode, os.environ.get('TRUTHFILE'))
u7, u8, u9 = (np.spacing(abs(Ns[i])) for i in (7, 8, 9))
k7 = [-1, 0, 1]; k8 = np.arange(-32, 33, 2); k9 = np.arange(-16384, 16385, 512)
H = np.array([[Ns[7]+a*u7, Ns[8]+b_*u8, Ns[9]+c*u9] for a in k7 for b_ in k8 for c in k9])
t0 = time.time(); v = d.obj('N', 7, H); i = int(np.argmax(v))
print('grid %d hyps %.0fs: best %.1f at dN7 %+d dN8 %+d dN9 %+d (median %.1f, sd %.1f)' % (len(H), time.time()-t0, v[i], k7[i//(len(k8)*len(k9))], k8[(i//len(k9)) % len(k8)], k9[i % len(k9)], np.median(v), v.std()))
best = H[i].copy()
for step8, step9 in [(1, 128), (1, 32)]:
    Hl = np.array([[best[0]+a*u7, best[1]+b_*step8*u8, best[2]+c*step9*u9] for a in (-1, 0, 1) for b_ in range(-3, 4) for c in range(-4, 5)])
    vl = d.obj('N', 7, Hl); j = int(np.argmax(vl)); best = Hl[j]; print('  refine: %.1f' % vl[j])
N1 = Ns.copy(); N1[7:] = best
open(out, 'w').write(' '.join(v_.hex() for v_ in N1)+'\n'+' '.join(v_.hex() for v_ in Ds)+'\n')
if mode == 'control':
    Nt, Dt = ld(os.environ['TRUTHFILE']); vt = d.obj('N', 7, np.array([Nt[7:]]))[0]
    print('  truth objective %.1f; found - truth (ulps): N7 %+d N8 %+d N9 %+d' % (vt, *[round((best[k]-Nt[7+k])/np.spacing(abs(Nt[7+k]))) for k in range(3)]))

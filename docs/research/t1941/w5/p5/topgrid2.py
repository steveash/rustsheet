"""N-chain top in three steps (T-1941 W5 part 5):
 1. (N8, N9) grid on the exact S-only objective (S8, P7, S7; independent of N7's whole-u shifts), coarse then fine;
 2. N7 integer via the FFT m-scan on P6 (mscan.py);
 3. exact 3-D local refine on the full objective (with P6).
usage: topgrid2.py control|real START OUT   env: BASE, TRUTHFILE, R8, S8, R9, S9, M"""
import numpy as np, sys, os, time
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import Data, ld
from mscan import mscan
mode, start, out = sys.argv[1], sys.argv[2], sys.argv[3]
N0, D0 = ld(os.environ.get('BASE', os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))); Ns, Ds = ld(start)
d = Data(N0, D0, mode, os.environ.get('TRUTHFILE'))
u7, u8, u9 = (float(np.spacing(abs(Ns[i]))) for i in (7, 8, 9))
R8, S8, R9, S9 = int(os.environ.get('R8', 256)), int(os.environ.get('S8', 2)), int(os.environ.get('R9', 32768)), int(os.environ.get('S9', 256))
t0 = time.time()
k8 = np.arange(-R8, R8+1, S8); k9 = np.arange(-R9, R9+1, S9)
H = np.array([[Ns[7], Ns[8]+a*u8, Ns[9]+c*u9] for a in k8 for c in k9]); v = d.obj('N', 7, H, 0)
o = np.argsort(v)[::-1]; mu, sd = np.median(v), v.std()
print('step 1 coarse: %d hyps %.0fs; top: %s' % (len(H), time.time()-t0, ' | '.join('dN8 %+d dN9 %+d %.0f (Z %.1f)' % (k8[i//len(k9)], k9[i % len(k9)], v[i], (v[i]-mu)/sd) for i in o[:4])), flush=True)
best = H[o[0]].copy()
Hf = np.array([[Ns[7], best[1]+a*u8, best[2]+c*u9] for a in range(-3, 4) for c in range(-S9, S9+1, 16)]); vf = d.obj('N', 7, Hf, 0)
best = Hf[int(np.argmax(vf))]; print('step 1 fine: %.0f' % vf.max(), flush=True)
M = int(float(os.environ.get('M', 2e4)))
ms, om, _ = mscan(d, Ns[7], best[1], best[2], M=M); im = np.argsort(om)[::-1]
print('step 2 m-scan top: %s (sd %.1f)' % (' '.join('%+d:%.1f' % (ms[i], om[i]) for i in im[:5]), om.std()), flush=True)
# step 3: exact refine around the top 5 m candidates
cands = []
for i in im[:5]:
    b7 = Ns[7]+ms[i]*u7
    Hr = np.array([[b7+a*u7, best[1]+bb*u8, best[2]+c*u9] for a in (-1, 0, 1) for bb in range(-3, 4) for c in range(-256, 257, 32)])
    vr = d.obj('N', 7, Hr, 1); j = int(np.argmax(vr)); cands.append((vr[j], Hr[j]))
cands.sort(key=lambda t: -t[0]); top = cands[0][1]
print('step 3 refine: %s' % ' | '.join('%.0f' % c[0] for c in cands), flush=True)
N1 = Ns.copy(); N1[7:] = top
open(out, 'w').write(' '.join(x_.hex() for x_ in N1)+'\n'+' '.join(x_.hex() for x_ in Ds)+'\n')
if mode == 'control':
    Nt, Dt = ld(os.environ['TRUTHFILE'])
    print('found - truth (ulps): N7 %+d N8 %+d N9 %+d; truth full objective %.0f' % (*[round((top[k]-Nt[7+k])/np.spacing(abs(Nt[7+k]))) for k in range(3)], d.obj('N', 7, np.array([Nt[7:]]), 1)[0]))

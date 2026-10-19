"""Last step: N0 by exact-match rate once everything else is solved (T-1941 W5 part 5).
usage: n0scan.py control|real IN OUT   env: TRUTHFILE (control), KR (range), NSUB"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from hypolib import ld
from sc import res, score_batch
mode, fin, fout = sys.argv[1:4]
z = np.load(os.environ.get('ROWS', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows3.txt')+'.near1.npz')
x = z['x']; b = z['b']
if mode == 'control': b = res(x, *ld(os.environ['TRUTHFILE']))
rng = np.random.default_rng(0); i = rng.choice(len(x), int(os.environ.get('NSUB', 30000)), replace=False); xs = x[i]; bs = b[i]
N, D = ld(fin); u = np.spacing(abs(N[0])); KR = int(float(os.environ.get('KR', 2e6)))
def best(ks):
    P = np.repeat(N[None], len(ks), 0); P[:, 0] = N[0]+ks*u
    s = score_batch(xs, bs, P, np.repeat(D[None], len(ks), 0)); j = int(np.argmax(s)); return ks[j], s[j], s
k1, s1, _ = best(np.arange(-KR, KR+1, 256))
k2, s2, _ = best(np.arange(k1-512, k1+513))
N[0] += k2*u; full = np.mean(res(x, N, D) == b)
print('N0 coarse best k %+d (%.4f), fine k %+d (%.4f); full-data exact %.4f' % (k1, s1, k2, s2, full))
if mode == 'control':
    Nt, _ = ld(os.environ['TRUTHFILE']); print('   N0 - truth: %+d ulps' % round((N[0]-Nt[0])/u))
open(fout, 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')

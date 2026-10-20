"""Is K3's D0 exactly 2^-13? (T-1941 W5 part 5; after W6 found D0 != power of two on (1.5,2]). Scan D0 + k ulps
by exact-match rate on real data with everything else fixed (D0's next op is the division, like N0).
usage: d0scan.py SET [NPZ]   env: NSUB (200000), KR (coarse range), STEP (coarse step), JOINT (also re-scan N0)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from hypolib import ld
from sc import score_batch, res
N, D = ld(sys.argv[1])
z = np.load(sys.argv[2] if len(sys.argv) > 2 else '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
i = np.random.default_rng(0).choice(len(z['x']), int(os.environ.get('NSUB', 200000)), replace=False); x, b = z['x'][i], z['b'][i]
u = float(np.spacing(D[0])); KR = int(float(os.environ.get('KR', 2e6))); ST = int(os.environ.get('STEP', 64))
def sc(ks):
    Q = np.repeat(D[None], len(ks), 0); Q[:, 0] = D[0]+ks*u
    return score_batch(x, b, np.repeat(N[None], len(ks), 0), Q)
ks = np.arange(-KR, KR+1, ST); s = sc(ks); o = np.argsort(s)[::-1]
print('D0 = %s, ulp %.3g; base (k=0) exact %.4f' % (D[0].hex(), u, sc(np.array([0]))[0]))
print('coarse (step %d, +-%d): top %s; median %.4f' % (ST, KR, ' '.join('%+d:%.4f' % (ks[j], s[j]) for j in o[:6]), np.median(s)))
k1 = ks[o[0]]; kf = np.arange(k1-ST, k1+ST+1); sf = sc(kf); j = int(np.argmax(sf))
print('fine around %+d: best k %+d exact %.4f (D0 %s)' % (k1, kf[j], sf[j], (D[0]+kf[j]*u).hex()))

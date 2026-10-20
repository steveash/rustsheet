"""Structure test by climbing (T-1941 W5 part 5): from a start set, greedy coordinate climb of the real exact rate
over chosen coefficients (steps of +-1,2,4,...,2^K ulps), under a given set of unstored ops. A true structure should
climb clearly higher than the all-stored one from the same start.
usage: structclimb.py START OPS OUT   env: NSUB (200000), COEFS (N0..N6,D1..D6), K (12), ROUNDS (3)"""
import numpy as np, sys, os, time
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1]); ops = [o for o in sys.argv[2].split(',') if o]; out = sys.argv[3]
z = np.load('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
i = np.random.default_rng(2).choice(len(z['x']), int(os.environ.get('NSUB', 200000)), replace=False); x, b = z['x'][i], z['b'][i]
xl = L(x); t = L(st(xl-L(1.0)))
sk = {'N': set(), 'D': set()}
for o in ops: ch, k = o.split('.'); sk[ch].add((k[0], int(k[1:])))
def chain(C, skip):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1):
        p = s*xl; p = p if ('P', j) in skip else L(st(p))
        s = p+L(C[j]); s = s if ('S', j) in skip else L(st(s))
    return s
def score(): return float(np.mean(st(t*L(st(chain(N, sk['N'])/chain(D, sk['D'])))) == b))
coefs = os.environ.get('COEFS', 'N0,N1,N2,N3,N4,N5,N6,D1,D2,D3,D4,D5,D6').split(',')
K = int(os.environ.get('K', 12)); best = score(); t0 = time.time()
print('ops %s start %.4f' % (ops or 'none', best), flush=True)
for rnd in range(int(os.environ.get('ROUNDS', 3))):
    for c in coefs:
        C = N if c[0] == 'N' else D; j = int(c[1:]); u = float(np.spacing(abs(C[j]))); moved = 0
        improved = True
        while improved:
            improved = False
            for s in [sg*2**e for e in range(K+1) for sg in (1, -1)]:
                C[j] += s*u; v = score()
                if v > best: best = v; moved += s; improved = True; break
                C[j] -= s*u
        if moved: print('  r%d %s %+d -> %.4f (%.0fs)' % (rnd, c, moved, best, time.time()-t0), flush=True)
print('final %.4f' % best)
open(out, 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')

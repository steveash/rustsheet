"""Exact-count climb in the K3 near-null lattice (T-1941 W5 part 5). From START, each step tries +-1, +-2 of every
basis vector and +-v_i +- v_j for all pairs; takes the best gain on a training sample and reports a disjoint held-out
sample. usage: k3climb.py START BASIS.npy OUT   env: NTR (300000), NHO (300000), STEPS (40)"""
import numpy as np, sys, os, itertools
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from hypolib import ld
from sc import score_batch
N, D = ld(sys.argv[1]); V = np.load(sys.argv[2]); out = sys.argv[3]
idx = [('N', i) for i in range(3, 10)] + [('D', i) for i in range(3, 10)]
z = np.load('/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows6.npz')
p = np.random.default_rng(21).permutation(len(z['x'])); ntr, nho = int(os.environ.get('NTR', 300000)), int(os.environ.get('NHO', 300000))
xt, bt = z['x'][p[:ntr]], z['b'][p[:ntr]]; xh, bh = z['x'][p[ntr:ntr+nho]], z['b'][p[ntr:ntr+nho]]
def apply(N, D, mv):
    N2, D2 = N.copy(), D.copy()
    for (c, i), d in zip(idx, mv):
        C = N2 if c == 'N' else D2; C[i] += d*np.spacing(abs(C[i]))
    return N2, D2
moves = [s*V[i] for i in range(len(V)) for s in (1, -1, 2, -2)] + [si*V[i]+sj*V[j] for i, j in itertools.combinations(range(len(V)), 2) for si in (1, -1) for sj in (1, -1)]
def sc(N, D, x, b, mvs):
    P, Q = zip(*[apply(N, D, m) for m in mvs]); return score_batch(x, b, np.array(P), np.array(Q))
cur = sc(N, D, xt, bt, [np.zeros(len(idx), int)])[0]; ho = sc(N, D, xh, bh, [np.zeros(len(idx), int)])[0]
print('start: train %.5f held-out %.5f' % (cur, ho), flush=True)
total = np.zeros(len(idx), int)
for step in range(int(os.environ.get('STEPS', 40))):
    s = sc(N, D, xt, bt, moves); j = int(np.argmax(s))
    if s[j] <= cur: print('local maximum (best try %.5f)' % s[j]); break
    N, D = apply(N, D, moves[j]); total += moves[j]; cur = s[j]
    ho = sc(N, D, xh, bh, [np.zeros(len(idx), int)])[0]
    print('step %2d: train %.5f held-out %.5f  move %s' % (step, cur, ho, ' '.join('%s%d:%+d' % (c, i, d) for (c, i), d in zip(idx, moves[j]) if d)), flush=True)
    open(out, 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')
print('total move: ' + ' '.join('%s%d:%+d' % (c, i, d) for (c, i), d in zip(idx, total) if d))

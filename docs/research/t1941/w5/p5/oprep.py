"""Per-op repair fingerprint on real data (T-1941 W5 part 5; after W2's K2 finding): for a set of unstored ops,
count rows repaired (miss -> hit) and broken (hit -> miss) against the all-stored model.
usage: oprep.py SET [NPZ]   env: NSUB (default all), COMBOS ('D.S3;D.S3,D.P3;...')"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1])
z = np.load(sys.argv[2] if len(sys.argv) > 2 else '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')
x, b = z['x'], z['b']
ns = int(os.environ.get('NSUB', 0))
if ns: i = np.random.default_rng(1).choice(len(x), ns, replace=False); x, b = x[i], b[i]
xl = L(x)
def chain(C, skip):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1):
        p = s*xl; p = p if ('P', j) in skip else L(st(p))
        s = p+L(C[j]); s = s if ('S', j) in skip else L(st(s))
    return s
def model(ops):
    sk = {o.split('.')[0]: set() for o in ('N.', 'D.')}
    for o in ops:
        ch, k = o.split('.'); sk.setdefault(ch, set()).add((k[0], int(k[1:])))
    a = L(st(chain(N, sk.get('N', set()))/chain(D, sk.get('D', set()))))
    return st(L(st(xl-L(1.0)))*a)
h0 = model([]) == b
print('rows %d, base exact %.4f' % (len(x), h0.mean()), flush=True)
combos = os.environ.get('COMBOS') or ';'.join('%s.%s%d' % (c, k, j) for c in 'ND' for k in 'PS' for j in range(9))
out = []
for cb in combos.split(';'):
    h = model(cb.split(',')) == b; rep = int((h & ~h0).sum()); brk = int((~h & h0).sum())
    out.append((rep-brk, cb, rep, brk))
for net, cb, rep, brk in sorted(out, reverse=True):
    print('%-24s repaired %7d broken %7d net %+7d (%+.4f)' % (cb, rep, brk, net, net/len(x)))

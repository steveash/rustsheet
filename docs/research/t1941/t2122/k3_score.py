"""Score K3 sets on a rows npz (w5/p5/mkrows.py) with the exact x87 scorer (w5/libsc.so), all / |x-1|>2e-3 / near 1 (T-2122 run 3). usage: k3_score.py W5DIR ROWS.npz SET..."""
import sys, numpy as np
sys.path.insert(0, sys.argv[1])
from sc import score
z = np.load(sys.argv[2]); x, b = z['x'], z['b']
def ld(f):
    l = open(f).read().split('\n'); return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]
far = np.abs(x-1) > 2e-3
for f in sys.argv[3:]:
    N, D = ld(f)
    print('%-12s all %.4f (%d pts)  |x-1|>2e-3 %.4f  near-1 %.4f' % (f.split('/')[-1], score(x, b, N, D), len(x),
          score(np.ascontiguousarray(x[far]), np.ascontiguousarray(b[far]), N, D), score(np.ascontiguousarray(x[~far]), np.ascontiguousarray(b[~far]), N, D)))

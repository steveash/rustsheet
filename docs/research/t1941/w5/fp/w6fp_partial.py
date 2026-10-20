"""(1.5,2] fingerprint, partial masks (same method as w6fp.py; calibration with xl_chain_d3ls and xl_chain_80)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); W6 = os.path.join(HERE, '..', '..', 'w6')
sys.path.insert(0, HERE); sys.path.insert(0, W6); os.environ.setdefault('KERNEL', 'k6')
from fp import L, fit, stats
import kern, synth
rows = [l.split() for l in open(sys.argv[1])]
X = np.array([float.fromhex(r[0]) for r in rows]); LO = np.array([float.fromhex(r[2]) for r in rows]); HI = np.array([float.fromhex(r[3]) for r in rows])
m = (X > 1.5) & (X < 1.98); X, LO, HI = X[m], LO[m], HI[m]
def rstar(lo, hi): return (L(lo)+L(hi))/L(2)
def run(Rs, xx):
    ok = np.isfinite(Rs.astype(float)); _, _, r = fit(xx[ok], Rs[ok], 9, 9, 1.5, 2.0); r = r[np.abs(r) < 20]; return stats(r)
MASKS = {'stored': [], 'final S0 both': ['N.S0', 'D.S0'], 'final P0,S0 both': ['N.P0', 'N.S0', 'D.P0', 'D.S0'],
         'N low (P0..S1)': ['N.P1', 'N.S1', 'N.P0', 'N.S0'], 'D low (P0..S1)': ['D.P1', 'D.S1', 'D.P0', 'D.S0'],
         'N low P0..S3': 'N.P3,N.S3,N.P2,N.S2,N.P1,N.S1,N.P0,N.S0'.split(','),
         'MASK_80': 'N.P3,N.S3,N.P2,N.S2,N.P1,N.S1,N.P0,N.S0,D.S2,D.P1,D.S1,D.P0,D.S0'.split(',')}
for seed in [int(s) for s in os.environ.get("SEEDS", "2,3").split(",")]:
    i = np.random.default_rng(seed).choice(len(X), 200000, replace=False); x, lo, hi = X[i], LO[i], HI[i]
    rr = run(rstar(lo, hi), x); print('seed %d real: rms %.3f median %.3f' % (seed, *rr), flush=True)
    for cf in ('xl_chain_d3ls.txt', 'xl_chain_80.txt'):
        N, D = synth.read_chain(os.path.join(W6, 'data', cf))
        for nm, keep in MASKS.items():
            R = synth.simR(N, D, x, keep); y = kern.outer(x, R); l2, h2, _ = kern.extract(x, y, 5); s = run(rstar(l2, h2), x)
            print('   %-15s %-18s rms %.3f median %.3f   (real - sim median %+.3f)' % (cf[9:-4], nm, *s, rr[1]-s[1]), flush=True)

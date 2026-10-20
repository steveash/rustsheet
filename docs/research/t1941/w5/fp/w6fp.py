"""(1.5,2] structure fingerprint (W6's kernel: y = st(L(x-1) L(st(L(x-2) L(R)))), R = st(N/D), raw-x (9,9) Horner;
W6's synth.simR / kern.outer / kern.extract used verbatim). Target R* = mean of the doubles R reproducing y (Rall's
Rlo..Rhi for real data, kern.extract for simulations). LS (9,9) Chebyshev fit of N - R* D on [1.5, 2]; residual rms in
ulp(R). Calibration: W6's two chains, each simulated all-stored and under W6's masks. usage: w6fp.py RALL [NSUB] [XHI]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); W6 = os.path.join(HERE, '..', '..', 'w6')
sys.path.insert(0, HERE); sys.path.insert(0, W6)
os.environ.setdefault('KERNEL', 'k6')
from fp import L, fit, stats
import kern, synth
ns = int(sys.argv[2]) if len(sys.argv) > 2 else 200000; xhi = float(sys.argv[3]) if len(sys.argv) > 3 else 1.98
rows = [l.split() for l in open(sys.argv[1])]
x = np.array([float.fromhex(r[0]) for r in rows]); lo = np.array([float.fromhex(r[2]) for r in rows]); hi = np.array([float.fromhex(r[3]) for r in rows])
m = (x > 1.5) & (x < xhi); x, lo, hi = x[m], lo[m], hi[m]
i = np.random.default_rng(int(os.environ.get('SEED', 1))).choice(len(x), min(ns, len(x)), replace=False); x, lo, hi = x[i], lo[i], hi[i]
def rstar(lo, hi): return (L(lo)+L(hi))/L(2)
def run(Rs, xx):
    ok = np.isfinite(Rs.astype(float)); _, _, r = fit(xx[ok], Rs[ok], 9, 9, 1.5, 2.0); r = r[np.abs(r) < 20]; return stats(r)
print('(1.5,%.2f) %d points' % (xhi, len(x)))
print('  real                         rms %.3f median %.3f' % run(rstar(lo, hi), x), flush=True)
M80 = 'N.P3,N.S3,N.P2,N.S2,N.P1,N.S1,N.P0,N.S0,D.S2,D.P1,D.S1,D.P0,D.S0'.split(',')
M51 = 'N.P3,N.S3,N.P2,N.S2,N.P1,N.S1,N.P0,N.S0,D.P1,D.S1,D.P0,D.S0'.split(',')
for cf in ('xl_chain_80.txt', 'xl_chain_d3ls.txt'):
    N, D = synth.read_chain(os.path.join(W6, 'data', cf))
    for nm, keep in (('all stored', []), ('MASK_80', M80), ('mask 51.3%', M51)):
        R = synth.simR(N, D, x, keep); y = kern.outer(x, R); l2, h2, n2 = kern.extract(x, y, 5)
        print('  sim %-18s %-11s rms %.3f median %.3f' % (cf[:-4], nm, *run(rstar(l2, h2), x)), flush=True)

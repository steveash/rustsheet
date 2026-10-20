"""Hand-off model test on K3 (T-1941 W5 part 5; W2's K2 model): the stored N.S_JN and D.S_JD are
fl(smooth + delta), with smooth = the exact upper polynomial (no stage roundings) and delta constant per run,
then the exact stored chain below. Per run, scan (dn, dd) on a grid of u/Q over +-R ulps; report the best per-run
hit rate against plain Horner (all stored) and the 1-ulp-shifted null.
usage: handoff.py SET JN JD   env: NRUN (24), RL (2048), Q (8), R (4)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1]); JN, JD = int(sys.argv[2]), int(sys.argv[3])
Q, R = int(os.environ.get('Q', 8)), int(os.environ.get('R', 4)); RL = int(os.environ.get('RL', 2048))
z = np.load(os.environ.get('NPZ', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')); X, B = z['x'], z['b']
br = np.where(np.nextafter(X[:-1], np.inf) != X[1:])[0]+1
segs = [s[:RL] for s in np.split(np.arange(len(X)), br) if len(s) >= 512]
segs = [segs[i] for i in np.linspace(0, len(segs)-1, int(os.environ.get('NRUN', 24))).astype(int)]
def smooth(C, J, xl):        # exact-ish upper polynomial sum_{j>=J} C_j x^(j-J) in long double
    s = L(0)
    for j in range(len(C)-1, J-1, -1): s = s*xl+L(C[j])
    return s
def lower(C, J, sJ, xl):
    s = L(sJ)
    for j in range(J-1, -1, -1): s = L(st(L(st(s*xl))+L(C[j])))
    return s
def horner(C, xl):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1): s = L(st(L(st(s*xl))+L(C[j])))
    return s
ds = np.arange(-R*Q, R*Q+1)/Q
print('run x          plain  best-handoff (dn, dd)   null-best')
for s in segs:
    x, b = X[s], B[s]; xl = L(x); t = L(st(xl-L(1.0))); bn = np.nextafter(b, np.inf)
    plain = np.mean(st(t*L(st(horner(N, xl)/horner(D, xl)))) == b)
    sn, sd = smooth(N, JN, xl), smooth(D, JD, xl)
    un, ud = L(np.spacing(np.abs(sn.astype(float)))), L(np.spacing(np.abs(sd.astype(float))))
    LN = [lower(N, JN, st(sn+L(a)*un), xl) for a in ds]; LD = [lower(D, JD, st(sd+L(c)*ud), xl) for c in ds]
    best = (0, 0, 0); bestn = 0
    for i, a in enumerate(LN):
        for k, c in enumerate(LD):
            y = st(t*L(st(a/c))); h = np.mean(y == b); hn = np.mean(y == bn)
            if h > best[0]: best = (h, ds[i], ds[k])
            bestn = max(bestn, hn)
    print('%.9f  %.3f  %.3f (%+.3f, %+.3f)   %.3f' % (x[0], plain, *best, bestn), flush=True)

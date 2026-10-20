"""Per-run sub-ulp offset meter (T-1941 W5 part 5; HANDOFF §12.3 step 1 adapted to K3): stored Horner chain, with an
offset delta (on a u/Q grid, |delta| <= u/2) added to the pre-rounding sums P_JN + N_JN and P_JD + D_JD. Per run,
the (delta_N, delta_D) with the best exact rate. A smooth coefficient offset upstream shows up as a run-dependent
delta that lifts the run's exact rate well above delta = 0.
usage: deltameter.py SET JN JD   env: NRUN (24), RL (4096), Q (32), NPZ, TRUTHFILE (control: B from that set)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
from meter import L, st
N, D = ld(sys.argv[1]); JN, JD = int(sys.argv[2]), int(sys.argv[3]); Q = int(os.environ.get('Q', 32)); RL = int(os.environ.get('RL', 4096))
z = np.load(os.environ.get('NPZ', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows4.txt.near1.npz')); X, B = z['x'], z['b']
br = np.where(np.nextafter(X[:-1], np.inf) != X[1:])[0]+1
segs = [s[:RL] for s in np.split(np.arange(len(X)), br) if len(s) >= 1024]
segs = [segs[i] for i in np.linspace(0, len(segs)-1, int(os.environ.get('NRUN', 24))).astype(int)]
def chain(C, xl, J, dl):
    s = L(C[-1])
    for j in range(len(C)-2, -1, -1):
        p = L(st(s*xl)); v = p+L(C[j])
        if j == J: v = v+dl*L(np.spacing(np.abs(st(v))))
        s = L(st(v))
    return s
ds = (np.arange(Q)-Q//2)/Q
if os.environ.get('TRUTHFILE'):
    Nt, Dt = ld(os.environ['TRUTHFILE'])
print('x0           base   best   (dN, dD)       2nd-best  null(B+1ulp)')
out = []
for s in segs:
    x = X[s]; xl = L(x); t = L(st(xl-L(1.0)))
    b = st(t*L(st(chain(Nt, xl, -1, 0)/chain(Dt, xl, -1, 0)))) if os.environ.get('TRUTHFILE') else B[s]
    bn = np.nextafter(b, np.inf)
    NN = [chain(N, xl, JN, L(a)) for a in ds]; DD = [chain(D, xl, JD, L(c)) for c in ds]
    R = np.array([[np.mean(st(t*L(st(n/d))) == b) for d in DD] for n in NN])
    Rn = max(np.mean(st(t*L(st(n/d))) == bn) for n in NN[::4] for d in DD[::4])
    f = np.sort(R.ravel())[::-1]; i, k = np.unravel_index(np.argmax(R), R.shape)
    print('%.9f  %.3f  %.3f (%+.3f, %+.3f)   %.3f     %.3f' % (x[0], R[Q//2, Q//2], f[0], ds[i], ds[k], f[1], Rn), flush=True)
    out.append((x[0], R[Q//2, Q//2], f[0], ds[i], ds[k]))
o = np.array(out); print('mean: base %.3f best %.3f' % (o[:, 1].mean(), o[:, 2].mean()))

"""Per-site law validation for the ISMF phase meter (T-1941 W5 part 5).
Synthetic target: base set with mid coefficients moved; for each rounding site, compare the measured per-run phase
with the predicted (pre-rounding value change)/ulp mod 1. Reports per site: visibility (median contrast) and
phase agreement (circular rms of measured - predicted)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, '..'))
from sc import load, res
from inj import phase
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
N0, D0 = ld(os.path.join(HERE, '..', 'p4', 'ss_fc.txt'))
# function-preserving synthetic move: softest SVD directions of the relative function map, ~TSIZE ulps
_xs = np.linspace(0.7, 1.5, 200).astype(np.longdouble)
_Nv = np.polyval(N0[::-1].astype(np.longdouble), _xs); _Dv = np.polyval(D0[::-1].astype(np.longdouble), _xs)
_cols = [np.spacing(abs(N0[i]))*_xs**i/_Nv for i in range(len(N0))] + [-np.spacing(abs(D0[i]))*_xs**i/_Dv for i in range(1, len(D0))]
_M = np.array(_cols, dtype=float).T; _U, _s, _Vt = np.linalg.svd(_M, full_matrices=False)
_rng = np.random.default_rng(int(os.environ.get('SEED', 1)))
_mv = sum(_rng.normal()*_Vt[k]*(1.0 if k >= 15 else 0.0) for k in range(len(_s)))
_mv = np.round(_mv/np.abs(_mv).max()*float(os.environ.get('TSIZE', 1e5)))
Nt = N0.copy(); Dt = D0.copy()
for i in range(len(N0)): Nt[i] += _mv[i]*np.spacing(abs(N0[i]))
for k, i in enumerate(range(1, len(D0))): Dt[i] += _mv[len(N0)+k]*np.spacing(abs(D0[i]))
print('synthetic move (ulps):', _mv.astype(int).tolist(), ' max rel function change %.1e' % float(np.abs(_M@_mv).max()))
x, b, c = load()
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0]+1
segs = [s[:int(os.environ.get('RL', 1024))] for s in np.split(np.arange(len(x)), br) if len(s) >= int(os.environ.get('RL', 1024)) and abs(x[s[0]]-1) > 0.02]
rng = np.random.default_rng(0); segs = [segs[i] for i in sorted(rng.choice(len(segs), min(60, len(segs)), replace=False))]
def stagevals(C, xv):
    s = C[-1]; out = {}
    for j in range(len(C)-2, -1, -1):
        p = s*xv; out[('P', j)] = (p, s); s = p+C[j]; out[('S', j)] = (s, None)
    return out
def delta_pre(ch, j, kind, xv):
    """change of the exact pre-rounding value at the site (smooth part)"""
    C0 = N0 if ch == 'N' else D0; C1 = Nt if ch == 'N' else Dt
    def S(C, jj): return sum(C[i]*xv**(i-jj) for i in range(jj, len(C)))
    return (S(C1, j+1)-S(C0, j+1))*xv if kind == 'P' else (S(C1, j)-S(C0, j))
rows = []
for ch in 'ND':
    C0 = N0 if ch == 'N' else D0
    for j in range(len(C0)-2, -1, -1):
        for kind in 'PS':
            meas = []; pred = []; con = []
            for s in segs:
                xx = x[s]; tgt = res(xx, Nt, Dt); xm = xx[len(xx)//2]
                ph, pk, ctr = phase(xx, tgt, N0, D0, 0 if ch == 'N' else 1, j, 0 if kind == 'P' else 1)
                v = stagevals(C0, xm)[(kind, j)][0]; u = np.spacing(abs(v))
                meas.append(ph); pred.append((delta_pre(ch, j, kind, xm)/u) % 1.0); con.append(ctr)
            meas = np.array(meas); pred = np.array(pred); con = np.array(con)
            dd = (meas-pred+0.5) % 1-0.5; vis = con > 0.15
            rows.append((ch, j, kind, np.median(con), vis.mean(), np.sqrt(np.mean(dd[vis]**2)) if vis.any() else np.nan))
            print('%s.%s%d  median contrast %.2f  visible-runs %.2f  phase rms (visible) %.3f' % rows[-1][:1]+rows[-1][2:3]+rows[-1][1:2]+rows[-1][3:] if False else '%s.%s%d  median contrast %.2f  visible-runs %.2f  phase rms (visible) %.3f' % (ch, kind, j, rows[-1][3], rows[-1][4], rows[-1][5]), flush=True)

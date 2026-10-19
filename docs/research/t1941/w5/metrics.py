"""T-1941 W5 metrics for a (N, D) candidate: far exact rate, near-1 exact rate, per-run bias rms (ulps)."""
import numpy as np
from sc import load, score, res
x, b, c = load()
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
rid = np.zeros(len(x), int); rid[br] = 1; rid = np.cumsum(rid)
rng = np.random.default_rng(5)
far = np.where(np.abs(x-1) > 0.02)[0]; near = np.where((np.abs(x-1) < 1e-3) & (x != 1))[0]
fs = np.sort(rng.choice(far, 120000, replace=False)); ns = np.sort(rng.choice(near, 60000, replace=False))
XF, BF, RF = x[fs], b[fs], rid[fs]; XN, BN = x[ns], b[ns]
UF = np.spacing(np.abs(BF))
_, RFi = np.unique(RF, return_inverse=True); cnt = np.bincount(RFi)
def metrics(N, D, full=False):
    r = res(XF, N, D)
    ex = np.mean(r == BF)
    d = (BF - r)/UF
    m = np.bincount(RFi, d)/cnt
    bias = np.sqrt(np.mean(m[cnt > 100]**2))
    nr = score(XN, BN, N, D)
    out = dict(far=ex, near=nr, bias=bias)
    if full: out['all'] = score(x, b, N, D)
    return out

"""Phase-free spectral overlap between Excel's rounding residual and a candidate's, on far runs (>=2048 points)."""
import numpy as np
from sc import load, res, candidate
x, b, c = load()
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
segs = [s[:2048] for s in np.split(np.arange(len(x)), br) if len(s) >= 2048 and abs(x[s[0]]-1) > 0.02]
idx = np.concatenate(segs); X = x[idx]; Bv = b[idx]; R = len(segs)
_N = candidate('N3').astype(np.longdouble); _D = candidate('D3').astype(np.longdouble)
XL = X.astype(np.longdouble)
ref = (XL-1)*np.polyval(_N[::-1], XL)/np.polyval(_D[::-1], XL)
U = np.spacing(np.abs(Bv)).astype(np.longdouble)
def pw(v):
    v = v.reshape(R, 2048); v = v - v.mean(1, keepdims=True)
    P = np.abs(np.fft.rfft(v, axis=1))**2; P[:, :20] = 0; return P
Pd = pw(((Bv-ref)/U).astype(float))
def overlap(N, D, per_run=False):
    m = res(X, N, D); Pm = pw(((m-ref)/U).astype(float))
    o = (Pd*Pm).sum(1)/np.sqrt((Pd**2).sum(1)*(Pm**2).sum(1))
    return o if per_run else float(np.median(o))
if __name__ == '__main__':
    print('runs', R)
    print('c5 overlap', overlap(candidate('N3'), candidate('D3')))
    N = candidate('N3').copy(); N[5] *= 1+1e-12; print('c5 with N5*(1+1e-12)', overlap(N, candidate('D3')))
    N = candidate('N3').copy(); N[5] *= 1+1e-8; print('c5 with N5*(1+1e-8)', overlap(N, candidate('D3')))
    N = candidate('N3').copy(); N[5] *= 1+1e-5; print('c5 with N5*(1+1e-5)', overlap(N, candidate('D3')))

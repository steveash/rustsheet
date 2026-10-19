"""Data vs c5 exact-simulation line spectra on long far runs. Lists data peaks with no c5 counterpart.
Residual per run: (B - ref)/ulp(B), ref = c5 exact function (long double). Hann window, zero-padded FFT."""
import numpy as np, sys, os
import os; sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from sc import load, res, candidate
x, b, c = load()
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
L = int(os.environ.get('RUNLEN', 2048))
segs = [s[:L] for s in np.split(np.arange(len(x)), br) if len(s) >= L and abs(x[s[0]]-1) > 0.02]
N = candidate('N3'); D = candidate('D3')
NL = N.astype(np.longdouble); DL = D.astype(np.longdouble)
NF = 1 << 16
def peaks(r, thr=40):
    r = r - r.mean(); P = np.abs(np.fft.rfft(r*np.hanning(len(r)), NF))**2; P /= np.median(P)
    i = np.where((P[1:-1] > P[:-2]) & (P[1:-1] >= P[2:]) & (P[1:-1] > thr))[0] + 1
    return [(k/NF, P[k]) for k in i]
def resid(xx, bb):
    XL = xx.astype(np.longdouble); ref = (XL-1)*np.polyval(NL[::-1], XL)/np.polyval(DL[::-1], XL)
    return ((bb-ref)/np.spacing(np.abs(bb)).astype(np.longdouble)).astype(float)
if __name__ == '__main__':
    tot = 0; un = 0; rows = []
    for s in segs:
        xx = x[s]; bb = b[s]; sim = res(xx, N, D)
        pd = peaks(resid(xx, bb)); ps = peaks(resid(xx, sim), 10)
        fs = np.array([f for f, _ in ps]) if ps else np.array([9.])
        for f, p in pd:
            tot += 1
            if np.min(np.abs(fs-f)) > 3.0/L:
                un += 1; rows.append((xx[0], f, p))
    print('runs %d, data peaks %d, unmatched by c5 sim %d' % (len(segs), tot, un))
    for r in sorted(rows, key=lambda r: -r[2])[:40]: print('  x %.5f  f %.5f  P %.0f' % r)

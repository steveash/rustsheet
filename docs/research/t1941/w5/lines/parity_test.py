"""Test: make c5's D3 and/or N2 even (last mantissa bit 0) by +-1 ulp, optionally compensating on the next-higher
coefficient to keep the x=1 stage sums; report exact far/near and the D.S3 / N.S2 triangle-line amplitudes."""
import numpy as np, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res, score
from metrics import XF, BF, XN, BN
def amp(N, D, ch, J):
    C = N if ch == 'N' else D; out = []
    for s in PK.segs:
        xx = PK.x[s]; xm = xx[len(xx)//2]
        st, _, _ = stages(C, xm); P, S = [(P, S) for j, P, dP, S, dS in st if j == J][0]
        if np.frexp(S)[1] <= np.frexp(P)[1]: continue
        XL = xx.astype(np.longdouble); Sv = np.polyval(C.astype(np.longdouble)[J:][::-1], XL)
        E = np.exp(-2j*np.pi*np.mod(Sv/np.longdouble(2*ulp(S)), 1.0).astype(float))*np.hanning(len(xx))
        d = PK.resid(xx, PK.b[s]); m = PK.resid(xx, res(xx, N, D))
        Ad = np.dot(d-d.mean(), E); As = np.dot(m-m.mean(), E)
        out.append((abs(Ad), abs(As), abs(Ad-As)))
    o = np.array(out); return 'n=%d data %.1f sim %.1f |data-sim| %.1f' % (len(o), o[:, 0].mean(), o[:, 1].mean(), o[:, 2].mean())
N0 = PK.N.copy(); D0 = PK.D.copy()
def even(v, k): return np.nextafter(v, np.inf if k > 0 else -np.inf)
for tag, N, D in [('c5', N0, D0)] + [('D3%+d%s N2%+d' % (kd, ' comp' if cp else '', kn), None, None) for kd in (0, 1, -1) for kn in (0, 1, -1) for cp in (0, 1)]:
    if N is None:
        kd = int(tag[2:4]); kn = int(tag.split('N2')[1]); cp = 'comp' in tag
        if kd == 0 and kn == 0: continue
        N = N0.copy(); D = D0.copy()
        if kd: dd = even(D[3], kd)-D[3]; D[3] += dd; D[4] -= dd if cp else 0
        if kn: dn = even(N[2], kn)-N[2]; N[2] += dn; N[3] -= dn if cp else 0
    print('%-18s far %.4f near %.4f | D.S3 %s | N.S2 %s' % (tag, score(XF, BF, N, D), score(XN, BN, N, D), amp(N, D, 'D', 3), amp(N, D, 'N', 2)), flush=True)

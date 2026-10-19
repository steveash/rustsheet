"""General FFT scan of the integer whole-ulp shift m of a top sum S_J (any chain), on the next product P_{J-1}
(T-1941 W5 part 5; generalises mscan.py). Hyp: coefficients c_J..c_top (ascending)."""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from meter import L, st
def top_S(x, hyp):
    s = np.full(len(x), hyp[-1])
    for c in hyp[-2::-1]:
        p = st(L(s)*L(x)); s = st(L(p)+L(c))
    return s
def mscan2(d, CH, J, hyp, M=20000, H=6):
    x = d.x; S = top_S(x, hyp); u = float(np.spacing(abs(np.median(S))))
    v = L(S)*L(x); uP = np.spacing(np.abs(st(v))).astype(L)
    F = d.weights(CH, J, J)[0]
    a = F*d.r*uP.astype(float)
    th0 = (v/uP); th0 = (th0-np.floor(th0)).astype(float)
    fr = (L(u)*L(x)/uP).astype(float)
    ms = np.arange(-M, M+1); out = np.zeros(len(ms)); NF = 1 << 24
    for h in range(1, H+1):
        z = a*((-1)**h/(np.pi*h))*np.exp(2j*np.pi*h*th0)
        k = np.round(h*fr*NF).astype(np.int64) % NF
        g = np.zeros(NF, complex); np.add.at(g, k, z)
        out += np.imag((np.fft.ifft(g)*NF)[ms % NF])
    return ms, out, u

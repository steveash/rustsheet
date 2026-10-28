"""Per-stage sawtooth-line phase read for K3 (T-1941 W5 part 5): shared by k3phase.py and ladder.py."""
import numpy as np, math, os
from fractions import Fraction as F
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
def ulp(v): a = np.abs(np.asarray(v).astype(np.float64)); return (np.nextafter(a, np.inf)-a).astype(LD)
KN = int(os.environ.get('KN', 99))  # T-2122 run 5: the N chain's ops j >= KN stay in 80-bit (maskmodel's mask)
def chain_off(C, xl, j0, off, kn=99):     # stored Horner (ops j >= kn in 80-bit); pre-rounding offset off*ulp at S_j0
    s = LD(C[-1])+0*xl
    for j in range(len(C)-2, -1, -1):
        v = (sd(s*xl) if j < kn else s*xl)+LD(C[j])
        if j == j0: v = v+LD(off)*ulp(sd(v))
        s = sd(v) if j < kn else v
    return s
def exact(xl, N0, D0): return (xl-LD(1))*np.polyval(N0.astype(LD)[::-1], xl)/np.polyval(D0.astype(LD)[::-1], xl)
def fsig(C, x0, j):                 # signed sum-line frequency of S_j at x0 (cycles per consecutive double)
    S = F(C[-1]); dS = F(0)
    for k in range(len(C)-2, j-1, -1):
        dS = dS*x0+S; S = S*x0+F(C[k])
    u = F(2)**(math.floor(math.log2(abs(float(S))))-52); ux = F(math.ulp(float(x0)))
    f = float((dS*ux/u) % 1); return f-1 if f > 0.5 else f
def amp(r, f): k = np.arange(len(r)); return np.dot(r-r.mean(), np.exp(-2j*np.pi*f*k))
def read_chunk(N0, D0, xs, y, stages, off=0.25):
    """Per-stage line phases of one chunk of consecutive doubles: list of (phase, amp ratio, calib shift, power, f)."""
    xl = xs.astype(LD); t = sd(xl-LD(1)); ex = exact(xl, N0, D0); uy = ulp(y)
    def res(yy): return np.asarray((yy.astype(LD)-ex)/uy, dtype=np.float64)
    def yof(n, d): return sd(t*sd(n/d)).astype(np.float64)
    nb, db = chain_off(N0, xl, -1, 0, KN), chain_off(D0, xl, -1, 0)
    rE, rS = res(y), res(yof(nb, db)); x0 = F(float(xs[len(xs)//2]))
    med = np.median(np.abs(np.fft.rfft(rS-rS.mean()))**2); out = []
    for ch, j in stages:
        C = N0 if ch == 'N' else D0; f = fsig(C, x0, j)
        rC = res(yof(chain_off(N0, xl, j, off, KN), db)) if ch == 'N' else res(yof(nb, chain_off(D0, xl, j, off)))
        AE, AS, AC = amp(rE, f), amp(rS, f), amp(rC, f)
        dC = np.angle(AC/AS)/(2*np.pi); sg = np.sign(dC) if abs(dC) > 0.02 else 0.0
        out.append((sg*np.angle(AE/AS)/(2*np.pi), abs(AE)/max(abs(AS), 1e-30), dC, abs(AS)**2/max(med, 1e-30), f))
    return out

"""K3 per-stage phase read (T-1941 W5 part 5; review 3's phase.py / phase_stages.py adapted to K3 and generalised).
Per run of consecutive doubles, the residual r = (y - exact)/ulp(y) carries each stored sum's rounding sawtooth as a
spectral line at f_j = S_j'(x) ulp(x) / ulp(S_j) cycles/step. The phase of Excel's line relative to the stored model's
(base set), signed by a planted +0.25-ulp offset at that sum, is the pre-rounding offset delta_j (ulp(S_j), mod 1).
Output per run and stage: x, phase, amplitude ratio |AE|/|AS|, calibration shift, line power / median.
usage: k3phase.py SET OUT.npy   env: MINLEN (4096), CH (8192 points per chunk), STAGES ("N8..N1,D8..D1")"""
import numpy as np, sys, os, math
from fractions import Fraction as F
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
LD = np.longdouble
def sd(v): return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)
def ulp(v): a = np.abs(np.asarray(v).astype(np.float64)); return (np.nextafter(a, np.inf)-a).astype(LD)
N0, D0 = ld(sys.argv[1])
def chain_off(C, xl, j0, off):     # stored Horner; pre-rounding offset off*ulp at sum S_j0
    s = LD(C[-1])+0*xl
    for j in range(len(C)-2, -1, -1):
        v = sd(s*xl)+LD(C[j])
        if j == j0: v = v+LD(off)*ulp(sd(v))
        s = sd(v)
    return s
def exact(xl): return (xl-LD(1))*np.polyval(N0.astype(LD)[::-1], xl)/np.polyval(D0.astype(LD)[::-1], xl)
def fsig(C, x0, j):                 # signed sum-line frequency of S_j at x0 (cycles per consecutive double)
    S = F(C[-1]); dS = F(0)
    for k in range(len(C)-2, j-1, -1):
        dS = dS*x0+S; S = S*x0+F(C[k])
    u = F(2)**(math.floor(math.log2(abs(float(S))))-52); ux = F(math.ulp(float(x0)))
    f = float((dS*ux/u) % 1); return f-1 if f > 0.5 else f
def amp(r, f): k = np.arange(len(r)); return np.dot(r-r.mean(), np.exp(-2j*np.pi*f*k))
z = np.load(os.environ.get('NPZ', '/tmp/claude-0/-home-user-rustsheet/3a20e203-66f6-532b-8608-5ba2c7a655ee/scratchpad/rows6.npz')); X, Y = z['x'], z['b']
br = np.where(np.nextafter(X[:-1], np.inf) != X[1:])[0]+1
MIN, CH = int(os.environ.get('MINLEN', 4096)), int(os.environ.get('CH', 8192))
chunks = [s[a:a+CH] for s in np.split(np.arange(len(X)), br) if len(s) >= MIN for a in range(0, len(s)-MIN+1, CH) if len(s[a:a+CH]) >= MIN]
chunks = [c for c in chunks if abs(X[c[0]]-1) > 1e-3]
stages = [(c[0], int(c[1:])) for c in os.environ.get('STAGES', 'N8,N7,N6,N5,N4,N3,N2,N1,D8,D7,D6,D5,D4,D3,D2,D1').split(',')]
out = []
for ci, c in enumerate(chunks):
    xl = X[c].astype(LD); y = Y[c]; t = sd(xl-LD(1)); ex = exact(xl); uy = ulp(y)
    def res(yy): return np.asarray((yy.astype(LD)-ex)/uy, dtype=np.float64)
    def yof(n, d): return sd(t*sd(n/d)).astype(np.float64)
    nb, db = chain_off(N0, xl, -1, 0), chain_off(D0, xl, -1, 0)
    rE, rS = res(y), res(yof(nb, db)); x0 = F(float(X[c[len(c)//2]]))
    med = np.median(np.abs(np.fft.rfft(rS-rS.mean()))**2)
    row = [float(X[c[len(c)//2]])]
    for ch, j in stages:
        C = N0 if ch == 'N' else D0; f = fsig(C, x0, j)
        rC = res(yof(chain_off(N0, xl, j, 0.25), db)) if ch == 'N' else res(yof(nb, chain_off(D0, xl, j, 0.25)))
        AE, AS, AC = amp(rE, f), amp(rS, f), amp(rC, f)
        dC = np.angle(AC/AS)/(2*np.pi); sg = np.sign(dC) if abs(dC) > 0.02 else 0.0
        row += [sg*np.angle(AE/AS)/(2*np.pi), abs(AE)/max(abs(AS), 1e-30), dC, abs(AS)**2/max(med, 1e-30), f]
    out.append(row)
    if ci % 50 == 0: print('chunk %d/%d x %.4f' % (ci, len(chunks), row[0]), flush=True)
np.save(sys.argv[2], np.array(out)); print('saved', len(out), 'chunks x', len(stages), 'stages')

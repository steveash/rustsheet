"""Global coherent scan of one top site's smooth phase function (T-1941 W5 part 5).

Model: relative to the base set, the site's pre-rounding value changes by a polynomial
    dS(x) = a + b*dx + c*dx^2 (dx = x - X0), in units of u = ulp of the site's stored value,
so the per-run phase is phi_r = (a + b*dx_r + c*dx_r^2) mod 1 (u constant over the scanned x range).
Per run r the correlation curve C_r(phi) (continuous template, meter.py) is expanded in harmonics h = 1..H;
for each c on a grid and each h, the sum over runs is an FFT over binned dx (non-uniform DFT in b); the best a per
(b, c) is taken analytically over a 64-point grid. Reports the top peaks with Z against the scan's own spread.
usage: scan.py control|real SITE (e.g. N.S7)   env: BMAX, CMAX, CSTEP, H, NB (FFT size log2)"""

import numpy as np, sys, os, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, ".."))
from sc import load, res
from meter import sites_and_weights, template, resid, L, PH

mode, site = sys.argv[1], sys.argv[2]
key = (site[0], site[2], int(site[3:]))


def ld(f):
    l = open(f).read().split("\n")
    return np.array([float.fromhex(v) for v in l[0].split()]), np.array(
        [float.fromhex(v) for v in l[1].split()]
    )


N0, D0 = ld(os.environ.get("BASE", os.path.join(HERE, "..", "p4", "ss_fc.txt")))
x, b, c = load()
X0 = 1.1
LO, HI = float(os.environ.get("LO", 0.7)), float(os.environ.get("HI", 1.47))
br = np.where(np.nextafter(x[:-1], np.inf) != x[1:])[0] + 1
RL = int(os.environ.get("RL", 4096))
segs = [
    s[:RL]
    for s in np.split(np.arange(len(x)), br)
    if len(s) >= 256 and LO <= x[s[0]] < HI and abs(x[s[0]] - 1) > 2e-3
]
if mode == "control":
    from synth import make_truth
    Nt, Dt, _mv, _fc = make_truth(N0, D0, int(os.environ.get('SEED', 1)), float(os.environ.get('TSIZE', 1e5)))
    print('synthetic move', _mv.astype(int).tolist(), 'func change %.1e' % _fc)
    tgt_all = None
truthabc = None
# per-run correlation curves
Cs = []
dxs = []
us = []
tph = []
t0 = time.time()
for s in segs:
    xx = x[s]
    W, ref = sites_and_weights(N0, D0, xx)
    B = res(xx, Nt, Dt) if mode == "control" else b[s]
    r = resid(xx, B, ref)
    pre, sv, w = W[key]
    T = template(pre, sv, w)
    Tc = T - T.mean(1, keepdims=True)
    nrm = np.sqrt((Tc * Tc).sum(1))
    Cr = (Tc @ r) / np.maximum(nrm, 1e-30) / max(np.sqrt(r @ r), 1e-30) * np.sqrt(len(xx))
    Cs.append(Cr)
    xm = xx[len(xx) // 2]
    dxs.append(xm - X0)
    us.append(float(np.median(np.spacing(np.abs(sv)))))
    if mode == "control":
        Wt, _ = sites_and_weights(Nt, Dt, xx)
        u = np.spacing(np.abs(sv)).astype(L)
        tph.append(float(np.median(((Wt[key][0] - pre) / u).astype(float))))
Cs = np.array(Cs)
dxs = np.array(dxs)
us = np.array(us)
R = len(Cs)
print("runs %d, prep %.0fs, distinct u: %s" % (R, time.time() - t0, np.unique(us)[:4]), flush=True)
if len(np.unique(us)) > 1:
    print("WARNING: u not constant over runs; restrict LO/HI")
u0 = np.median(us)
if mode == "control":
    # true polynomial in cycles: fit tph = a + b dx + c dx^2 (unwrapped, these are the exact smooth deltas)
    tp = np.polyfit(dxs, tph, 2)
    truthabc = (tp[2] % 1, tp[1], tp[0])
    print("truth (a mod 1, b, c) = (%.3f, %.2f, %.2f)" % truthabc)
def objective(a, bb, cc):
    ph = (a+bb*dxs+cc*dxs**2) % 1.0; k = np.floor(ph*64).astype(int) % 64; fr = ph*64-np.floor(ph*64)
    return float(np.sum(Cs[np.arange(R), k]*(1-fr)+Cs[np.arange(R), (k+1) % 64]*fr))
if os.environ.get('EVALONLY'):
    rng = np.random.default_rng(5)
    null = [objective(rng.random(), rng.uniform(-2e4, 2e4), rng.uniform(-200, 200)) for _ in range(20000)]
    print('null mean %.2f sd %.2f max(20k) %.2f' % (np.mean(null), np.std(null), np.max(null)))
    if truthabc: print('objective at truth %.2f  (Z %.1f)' % (objective(*truthabc), (objective(*truthabc)-np.mean(null))/np.std(null)))
    print('objective at (0,0,0) %.2f' % objective(0, 0, 0))
    sys.exit(0)
H = int(os.environ.get("H", 6))
Ch = (
    np.fft.fft(Cs, axis=1) / Cs.shape[1]
)  # C_r(phi_k) -> coefficients; C(phi) = sum_h Ch[h] e^{+2 pi i h phi}
BMAX = float(os.environ.get("BMAX", 2e4))
CMAX = float(os.environ.get("CMAX", 200))
CSTEP = float(os.environ.get("CSTEP", 0.05))
NB = 1 << int(os.environ.get("NB", 22))
span = dxs.max() - dxs.min() + 1e-9
dx0 = dxs.min()
cell = span / NB * 4  # bin size; b resolution = 1/(NB*cell)
kbin = np.round((dxs - dx0) / cell).astype(int)
bgrid = np.fft.fftfreq(NB, d=cell)  # cycles per unit dx
keepb = np.abs(bgrid) <= BMAX
best = []
cgrid = np.arange(-CMAX, CMAX + CSTEP / 2, CSTEP)
t0 = time.time()
ag = np.arange(64) / 64.0
for cc in cgrid:
    tot = np.zeros((64, keepb.sum()))
    for h in range(1, H + 1):
        z = np.zeros(NB, complex)
        np.add.at(z, kbin, Ch[:, h] * np.exp(2j * np.pi * h * (cc * dxs**2 + (dxs - dxs) * 0)))
        # sum_r z_r e^{2 pi i h b dx_r}: evaluate at b on bgrid via FFT at h*b -> use ifft on the binned array with dx offset
        G = np.fft.ifft(z) * NB  # G[m] = sum_k z_k e^{2 pi i k m / NB}
        # e^{2 pi i h b (dx0 + k cell)}; for b = f/(NB cell) with f integer and harmonic h we need index h*f
        f = np.round(bgrid[keepb] * NB * cell).astype(int)
        Gh = G[(h * f) % NB] * np.exp(2j * np.pi * h * bgrid[keepb] * dx0)
        tot += 2 * np.real(np.exp(2j * np.pi * h * ag)[:, None] * Gh[None, :])
    ia = tot.argmax(0)
    vals = tot[ia, np.arange(tot.shape[1])]
    top = np.argsort(vals)[-3:]
    for i in top:
        best.append((vals[i], ag[ia[i]], bgrid[keepb][i], cc))
best.sort(reverse=True)
allv = np.array([v[0] for v in best])
print("scan %.0fs; hypotheses %.2e" % (time.time() - t0, len(cgrid) * keepb.sum() * 64))
for v, a_, b_, c_ in best[:10]:
    print("  peak %.2f  a %.3f  b %.2f  c %.2f" % (v, a_, b_, c_))
if truthabc is not None:
    a_, b_, c_ = truthabc
    print("truth (a,b,c)=(%.3f, %.2f, %.2f)" % truthabc)

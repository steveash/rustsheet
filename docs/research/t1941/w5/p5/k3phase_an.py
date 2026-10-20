"""Analyse k3phase.py output: per stage, reliable chunks, neighbour coherence and circular law fits
ph(x) = A + B g(x) mod 1 for g in {x, 1/x, x^2, x^3} (B scanned; resultant R of exp(2 pi i (ph - B g)))."""
import numpy as np, sys
A = np.load(sys.argv[1]); stages = 'N8,N7,N6,N5,N4,N3,N2,N1,D8,D7,D6,D5,D4,D3,D2,D1'.split(',')
x = A[:, 0]
def circ(a, b): d = np.mod(a-b, 1.0); return np.minimum(d, 1-d)
gs = {'x': x, '1/x': 1/x, 'x^2': x**2, 'x^3': x**3}
rng = np.random.default_rng(0)
for k, sname in enumerate(stages):
    ph, ar, dC, pw, f = (A[:, 1+5*k+i] for i in range(5))
    ok = (np.abs(dC) > 0.12) & (ar > 0.5) & (ar < 2.0) & (pw > 4) & (np.abs(f) > 2/8192)
    if ok.sum() < 20: print('%s: %d reliable chunks (median power %.1f, median |dC| %.2f)' % (sname, ok.sum(), np.median(pw), np.median(np.abs(dC)))); continue
    xs, p = x[ok], ph[ok]; o = np.argsort(xs); xs, p = xs[o], p[o]
    adj = circ(p[1:], p[:-1]); rnd = circ(p[rng.permutation(len(p))], p)
    R0 = abs(np.exp(2j*np.pi*p).mean())
    best = []
    for gn, g in gs.items():
        gg = g[ok][o]; Bs = np.linspace(-400, 400, 64001)
        Rs = np.array([abs(np.exp(2j*np.pi*(p-B*gg)).mean()) for B in Bs[::40]]); i = np.argmax(Rs)
        Bf = Bs[max(0, i*40-60):i*40+60]; Rf = np.array([abs(np.exp(2j*np.pi*(p-B*gg)).mean()) for B in Bf]); j = np.argmax(Rf)
        best.append((Rf[j], gn, Bf[j], np.angle(np.exp(2j*np.pi*(p-Bf[j]*gg)).mean())/(2*np.pi)))
    best.sort(reverse=True)
    print('%s: %3d reliable | const-R %.2f | adj |dphi| %.3f (random %.3f) | best law: %s' % (sname, ok.sum(), R0, adj.mean(), rnd.mean(),
          '; '.join('%s B %+.2f A %+.2f R %.2f' % (gn, B, a, R) for R, gn, B, a in best[:2])))

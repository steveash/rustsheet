"""Local slopes of the per-stage offset laws from close chunk pairs (T-1941 W5 part 5). (1) Check: pairs with dx < 1e-6
should agree in phase if |slope| << 1e6. (2) Per x-window, the slope s maximising sum cos(2 pi (dphi - s dx)) over
pairs with 1e-5 < dx < 1e-2, |s| <= SMAX. usage: k3slope.py PHASES.npy   env: SMAX (30000), WIN (0.05)"""
import numpy as np, sys, os
A = np.load(sys.argv[1]); stages = 'N8,N7,N6,N5,N4,N3,N2,N1,D8,D7,D6,D5,D4,D3,D2,D1'.split(',')
x = A[:, 0]; SMAX, WIN = float(os.environ.get('SMAX', 3e4)), float(os.environ.get('WIN', 0.05))
def circ(d): d = np.mod(d, 1.0); return np.minimum(d, 1-d)
for sname in ('N7', 'N6', 'N3', 'D6', 'D5'):
    k = stages.index(sname); ph, ar, dC, pw, f = (A[:, 1+5*k+i] for i in range(5))
    ok = (np.abs(dC) > 0.12) & (ar > 0.5) & (ar < 2.0) & (pw > 4) & (np.abs(f) > 2/8192)
    xs, p = x[ok], ph[ok]; o = np.argsort(xs); xs, p = xs[o], p[o]
    I, J = np.triu_indices(len(xs), 1); dx = xs[J]-xs[I]; dp = p[J]-p[I]
    close = (dx > 1e-9) & (dx < 1e-6)
    rnd = circ(p[np.random.default_rng(0).permutation(len(p))][:close.sum()]-p[:close.sum()])
    print('%s: %d pairs dx<1e-6: mean |dphi| %.3f (random %.3f)' % (sname, close.sum(), circ(dp[close]).mean() if close.any() else np.nan, rnd.mean() if close.any() else np.nan))
    use = (dx > 1e-5) & (dx < 1e-2); s = np.arange(-SMAX, SMAX+1, 0.5)
    for lo in np.arange(0.70, 1.47, WIN):
        m = use & (xs[I] >= lo) & (xs[I] < lo+WIN) & (xs[J] < lo+WIN)
        if m.sum() < 30: continue
        sc = np.zeros(len(s))
        for c0 in range(0, len(s), 4000):
            sc[c0:c0+4000] = np.cos(2*np.pi*(dp[m][None, :]-s[c0:c0+4000, None]*dx[m][None, :])).sum(1)
        i = int(np.argmax(sc)); z = (sc[i]-sc.mean())/sc.std()
        sc2 = sc.copy(); sc2[max(0, i-200):i+200] = -1e9; i2 = int(np.argmax(sc2))
        print('   x %.2f-%.2f: %4d pairs, slope %+9.1f (Z %.1f, peak/m %.2f; 2nd %+9.1f Z %.1f)' % (lo, lo+WIN, m.sum(), s[i], z, sc[i]/m.sum(), s[i2], (sc[i2]-sc.mean())/sc.std()))

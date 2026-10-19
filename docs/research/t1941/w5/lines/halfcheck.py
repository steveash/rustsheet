"""Phase reliability of the D-chain tie lines: per run, data-vs-sim phase from the first and second halves.
Clean lines agree; contamination by nearby-frequency lines makes them disagree. Also synthetic control."""
import numpy as np, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res
N0 = PK.N.copy(); D0 = PK.D.copy(); D0[3] = np.nextafter(D0[3], -np.inf)
allidx = np.concatenate(PK.segs)
Dt = D0.copy(); rng = np.random.default_rng(11)
for i in (4, 5, 6, 7, 8): Dt[i] += int(rng.integers(-30000, 30000))*ulp(D0[i])
Bsyn = np.zeros_like(PK.x); Bsyn[allidx] = res(PK.x[allidx], N0, Dt)
for name, B in (('real', PK.b), ('synthetic', Bsyn)):
    per = {3: [], 4: [], 5: []}
    for s in PK.segs:
        xm = PK.x[s][len(s)//2]; st, _, _ = stages(D0, xm)
        for j, P, dP, S, dS in st:
            if j not in per or np.frexp(S)[1] != np.frexp(P)[1]+1: continue
            u2 = 2*ulp(S); xx = PK.x[s]
            Sv = np.polyval(D0.astype(np.longdouble)[j:][::-1], xx.astype(np.longdouble))
            ph = np.mod(Sv/np.longdouble(u2), 1.0).astype(float)
            d = PK.resid(xx, B[s]); m = PK.resid(xx, res(xx, N0, D0)); h = len(xx)//2; out = []
            for sl in (slice(0, h), slice(h, None)):
                E = np.exp(-2j*np.pi*ph[sl])*np.hanning(h); dd = d[sl]-d[sl].mean(); mm = m[sl]-m[sl].mean()
                out.append(np.angle(np.dot(dd, E)/np.dot(mm, E))/2/np.pi)
            per[j].append((out[0]-out[1]+0.5) % 1-0.5)
    print(name, ' '.join('D.S%d: n=%d half-diff rms %.3f' % (j, len(v), np.std(v)) for j, v in per.items()))

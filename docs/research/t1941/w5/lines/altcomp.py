"""Full-track test of T1/T2/T3 against rounding lines of alternative intermediates V (law f = fold(V'*h/ulp(V)))
and triangle variants (V landing k binades above its addend). V built from c5's exact polynomials."""
import numpy as np, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from predict import fold, ulp, stages
from sc import candidate
N = candidate('N3'); D = candidate('D3')
T1 = [(0.7413,0.3134),(0.7613,0.3195),(0.7813,0.3256),(0.8013,0.3319),(0.8213,0.3381),(0.8413,0.3445),(0.8711,0.3542),(0.9013,0.3642),(0.9213,0.3708),(0.9413,0.3776),(0.9613,0.3845),(0.9760,0.3896)]
T2 = [(0.7013,0.3515),(0.7213,0.3495),(0.7323,0.3481),(0.7497,0.3456),(0.7613,0.3437),(0.7813,0.3401),(0.7836,0.3396)]
T3 = [(1.3469,0.4668),(1.3613,0.4607),(1.3643,0.4594),(1.4013,0.4432),(1.4213,0.4344),(1.4413,0.4255),(1.4613,0.4164),(1.4813,0.4060)]
def funcs(x):
    stN, Nv, dN = stages(N, x); stD, Dv, dD = stages(D, x); t = x-1
    A = Nv/Dv; dA = (dN*Dv-Nv*dD)/Dv**2
    F = {'N': (Nv, dN), 'D': (Dv, dD), 'A': (A, dA), 'res': (t*A, A+t*dA), '1/D': (1/Dv, -dD/Dv**2),
         't*N': (t*Nv, Nv+t*dN), 't*D': (t*Dv, Dv+t*dD), 't/D': (t/Dv, 1/Dv-t*dD/Dv**2), 'x*A': (x*A, A+x*dA),
         'N/x': (Nv/x, dN/x-Nv/x**2), 'D/x': (Dv/x, dD/x-Dv/x**2)}
    for tag, st in (('N', stN), ('D', stD)):
        for j, P, dP, S, dS in st: F['%s.P%d' % (tag, j)] = (P, dP); F['%s.S%d' % (tag, j)] = (S, dS)
    return F
names = list(funcs(0.8).keys())
for tn, T in (('T1', T1), ('T2', T2), ('T3', T3)):
    best = []
    for n in names:
        for k in range(0, 4):          # k: landing k binades above (k=0 plain sawtooth/sum), law fold(V'h/(2^k ulp))
            for hm in (1, 2):         # harmonic
                d = []
                for x, f in T:
                    V, dV = funcs(x)[n]; d.append(abs(fold(hm*dV*ulp(x)/ulp(V)/2**k) - f))
                best.append((np.max(d), n, k, hm))
    best.sort()
    print(tn, ' '.join('%s/k%d/h%d:%.1e' % (n, k, hm, e) for e, n, k, hm in best[:5]))

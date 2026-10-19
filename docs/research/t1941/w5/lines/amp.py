"""Demodulate data and c5-sim residuals at a stage's predicted triangle frequency (law S'h/(2 ulp S)) per run.
usage: amp.py CHAIN J   -> per run: x, |data|, |sim|, phase(data/sim) in cycles"""
import numpy as np, sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import peaks as PK
from predict import ulp, stages
from sc import res
ch, J = sys.argv[1], int(sys.argv[2]); C = PK.N if ch == 'N' else PK.D
for s in PK.segs:
    xx = PK.x[s]; bb = PK.b[s]; xm = xx[len(xx)//2]
    st, _, _ = stages(C, xm); P, dP, S, dS = [(P, dP, S, dS) for j, P, dP, S, dS in st if j == J][0]
    if np.frexp(S)[1] <= np.frexp(P)[1]: continue
    # exact local phase: frac(S(x_k)/(2ulp(S))) evaluated per point (long double Horner of the stage)
    XL = xx.astype(np.longdouble); Cl = C.astype(np.longdouble)
    Sv = np.polyval(Cl[J:][::-1], XL)
    ph = np.mod(Sv/np.longdouble(2*ulp(S)), 1.0).astype(float)
    E = np.exp(-2j*np.pi*ph)*np.hanning(len(xx))
    d = PK.resid(xx, bb); d -= d.mean(); m = PK.resid(xx, res(xx, PK.N, PK.D)); m -= m.mean()
    Ad = np.dot(d, E)/len(xx)*2; As = np.dot(m, E)/len(xx)*2
    print('x %.4f |data| %.3f |sim| %.3f  phase(data/sim) %+.3f' % (xm, abs(Ad), abs(As), np.angle(Ad/As)/2/np.pi))

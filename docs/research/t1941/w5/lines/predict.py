"""Predicted line frequencies (cycles/step, folded to [0,0.5]) of every c5 stage at x, sawtooth and triangle laws.
saw  : product P_j = S_{j+1}*x rounded in its binade:      f = (dP/dK)/ulp(P)
tri  : sum S_j = P_j + c_j landing one binade above P_j:   f = (dS/dK)/ulp(S)   (W4's law)
div  : A = N/D;  res : t*A."""
import numpy as np, sys, os
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), '..'))
from sc import candidate
def fold(f): f = f % 1.0; return min(f, 1-f)
def ulp(v): return np.spacing(abs(v))
def stages(c, x):
    """returns list of (name, P, dP/dx, S, dS/dx) for Horner of c at x"""
    s = c[-1]; ds = 0.0; out = []
    for j in range(len(c)-2, -1, -1):
        P = s*x; dP = ds*x + s; S = P + c[j]; out.append((j, P, dP, S, dP)); s, ds = S, dP
    return out, s, ds
def predict(x, N=None, D=None):
    N = candidate('N3') if N is None else N; D = candidate('D3') if D is None else D
    h = ulp(x); rows = []
    stN, Nv, dN = stages(N, x); stD, Dv, dD = stages(D, x)
    for tag, st in (('N', stN), ('D', stD)):
        for j, P, dP, S, dS in st:
            rows.append(('%s.P%d saw' % (tag, j), fold(dP*h/ulp(P))))
            eP = np.frexp(P)[1]; eS = np.frexp(S)[1]
            if eS > eP: rows.append(('%s.S%d tri(+%d)' % (tag, j, eS-eP), fold(dS*h/ulp(S))))
            elif eS == eP: rows.append(('%s.S%d same' % (tag, j), fold(dS*h/ulp(S))))
    A = Nv/Dv; dA = (dN*Dv - Nv*dD)/Dv**2
    rows.append(('A div', fold(dA*h/ulp(A))))
    t = x-1; r = t*A; dr = A + t*dA
    rows.append(('res', fold(dr*h/ulp(r))))
    return rows
if __name__ == '__main__':
    for xs in sys.argv[1:]:
        x = float(xs); print('x =', x)
        for n, f in predict(x): print('   %-16s %.5f' % (n, f))

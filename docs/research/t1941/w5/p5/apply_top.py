"""Apply a top-stage scan result (T-1941 W5 part 5): Delta S_J(x) = (a + m + b*dx + c*dx^2)*u, dx = x - X0, for the N
chain top sum S_7 = N7 + N8 x + N9 x^2 (generalised: J and degree from the chain length). The integer m is free
(a whole-u shift of S_J leaves its rounding pattern unchanged); take the one nearest the base. Then refit the lower
coefficients (not the plateau-read ones) to restore the base's function.
usage: apply_top.py BASE OUT CHAIN J a b c [u]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from meter import L
base, out, CH, J, a, bb, cc = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), float(sys.argv[5]), float(sys.argv[6]), float(sys.argv[7])
X0 = 1.1
def ld(f):
    l = open(f).read().split('\n'); return np.array([float.fromhex(v) for v in l[0].split()]), np.array([float.fromhex(v) for v in l[1].split()])
N0, D0 = ld(base); N1 = N0.copy(); D1 = D0.copy(); C = N1 if CH == 'N' else D1
# stage value scale u at the range centre
s = C[-1]
for j in range(len(C)-2, J-1, -1): s = s*X0+C[j]
u = float(sys.argv[8]) if len(sys.argv) > 8 else float(np.spacing(abs(s)))
top = len(C)-1
if top-J != 2: raise SystemExit('this helper handles a quadratic top tail only (top - J = 2)')
g = cc*u; bl = (bb-2*X0*cc)*u          # Delta c_top, Delta c_{top-1}
C[top] += round(g/np.spacing(abs(C[top])))*np.spacing(abs(C[top]))
C[top-1] += round(bl/np.spacing(abs(C[top-1])))*np.spacing(abs(C[top-1]))
# Delta c_J = (a+m)u - X0*Delta c_{J+1} - X0^2*Delta c_{J+2}, m chosen to minimise |Delta c_J|
rest = -X0*bl-X0**2*g
m = round(-(a*u+rest)/u); dJ = (a+m)*u+rest
C[J] += round(dJ/np.spacing(abs(C[J])))*np.spacing(abs(C[J]))
READ = {('N', 0), ('N', 1), ('N', 2), ('D', 1), ('D', 2), ('D', 3)}
free = [(CH, i) for i in range(J) if (CH, i) not in READ] + [(o, i) for o in ('N', 'D') if o != CH for i in range(1, len(D0 if o == 'D' else N0)) if (o, i) not in READ]
xs = np.linspace(0.7, 1.5, 120).astype(L); tgt = np.polyval(N0.astype(L)[::-1], xs)/np.polyval(D0.astype(L)[::-1], xs)
for it in range(4):
    Nv = np.polyval(N1.astype(L)[::-1], xs); Dv = np.polyval(D1.astype(L)[::-1], xs)
    M = np.array([(np.spacing(abs((N1 if c == 'N' else D1)[i]))*xs**i/(Nv if c == 'N' else -Dv)).astype(float) for c, i in free]).T
    sol, *_ = np.linalg.lstsq(M/1e-18, ((tgt*Dv/Nv)-1).astype(float)/1e-18, rcond=None)
    for (c, i), v in zip(free, sol):
        CC = N1 if c == 'N' else D1; CC[i] += round(v)*np.spacing(abs(CC[i]))
f = np.abs((np.polyval(N1.astype(L)[::-1], xs)/np.polyval(D1.astype(L)[::-1], xs))/tgt-1).max()
open(out, 'w').write(' '.join(v.hex() for v in N1)+'\n'+' '.join(v.hex() for v in D1)+'\n')
print('applied top (a %.3f m %d b %.2f c %.2f, u %.3g): moves %s; function restored %.1e' % (a, m, bb, cc, u,
      ' '.join('%s%d:%+d' % (CH, i, round((C[i]-(N0 if CH == 'N' else D0)[i])/np.spacing(abs((N0 if CH == 'N' else D0)[i])))) for i in range(J, top+1)), float(f)))

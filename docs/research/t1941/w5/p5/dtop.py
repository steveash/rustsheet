"""Turn a cubic top-stage scan result (scan3.py) into the top coefficients c_J..c_{J+3} (T-1941 W5 part 5).
Delta S_J(x) = (a + m + b dx + c dx^2 + d dx^3) u_S, dx = x - X0, for S_J = c_J + c_{J+1} x + c_{J+2} x^2 + c_{J+3} x^3.
The whole-u shift m is free (S_J rounding pattern unchanged); it is set to keep c_J nearest the base.
usage: dtop.py BASE OUT CHAIN J a b c d   (no refit: lower coefficients are peeled afterwards)"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from hypolib import ld
base, out, CH, J = sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]); a, b, c, d = map(float, sys.argv[5:9])
X0 = 1.1
N, D = ld(base); C = N if CH == 'N' else D
if len(C)-1-J != 3: raise SystemExit('cubic top tail only (top - J = 3)')
s = C[-1]
for j in range(len(C)-2, J-1, -1): s = s*X0+C[j]
u = float(np.spacing(abs(s)))
# polynomial in dx -> polynomial in x: d dx^3 + c dx^2 + b dx + a, dx = x - X0
p = np.polynomial.Polynomial([a, b, c, d])
q = p(np.polynomial.Polynomial([-X0, 1.0]))   # coefficients in x, ascending
dc = np.pad(q.coef, (0, 4-len(q.coef)))*u
def mv(i, v): C[i] += round(v/np.spacing(abs(C[i])))*np.spacing(abs(C[i]))
for i in (3, 2, 1): mv(J+i, dc[i])
m = round(-dc[0]/u); mv(J, dc[0]+m*u)
open(out, 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')
B = ld(base)[0 if CH == 'N' else 1]
print('dtop: %s' % ' '.join('%s%d:%+d' % (CH, J+i, round((C[J+i]-B[J+i])/np.spacing(abs(B[J+i])))) for i in range(4)))

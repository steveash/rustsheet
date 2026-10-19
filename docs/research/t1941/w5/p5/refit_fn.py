"""Restore a set's function to a target set's function by least-squares integer moves of the free coefficients
(T-1941 W5 part 5). Free: chain CH coefficients below J (except EXCL) and the other chain's coefficients (except EXCL).
usage: refit_fn.py IN TARGET OUT CH J [EXCL e.g. N0,N1,N2,D1,D2,D3]"""
import numpy as np, sys, os
HERE = os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0, HERE)
from meter import L
from hypolib import ld
fin, ftg, fout, CH, J = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
EXCL = set((e[0], int(e[1:])) for e in (sys.argv[6].split(',') if len(sys.argv) > 6 and sys.argv[6] else []))
N1, D1 = ld(fin); Nt, Dt = ld(ftg)
other = 'D' if CH == 'N' else 'N'
free = [(CH, i) for i in range(J) if (CH, i) not in EXCL and not (CH == 'D' and i == 0)] + \
       [(other, i) for i in range(len(N1 if other == 'N' else D1)) if (other, i) not in EXCL and not (other == 'D' and i == 0)]
xs = np.linspace(0.7, 1.5, 160).astype(L); tgt = np.polyval(Nt.astype(L)[::-1], xs)/np.polyval(Dt.astype(L)[::-1], xs)
for it in range(5):
    Nv = np.polyval(N1.astype(L)[::-1], xs); Dv = np.polyval(D1.astype(L)[::-1], xs)
    M = np.array([(np.spacing(abs((N1 if c == 'N' else D1)[i]))*xs**i/(Nv if c == 'N' else -Dv)).astype(float) for c, i in free]).T
    sol, *_ = np.linalg.lstsq(M/1e-18, ((tgt*Dv/Nv)-1).astype(float)/1e-18, rcond=None)
    for (c, i), v in zip(free, sol):
        CC = N1 if c == 'N' else D1; CC[i] += round(v)*np.spacing(abs(CC[i]))
f = np.abs((np.polyval(N1.astype(L)[::-1], xs)/np.polyval(D1.astype(L)[::-1], xs))/tgt-1).max()
open(fout, 'w').write(' '.join(v.hex() for v in N1)+'\n'+' '.join(v.hex() for v in D1)+'\n')
print('refit %s -> %s: free %d, max rel function diff %.1e' % (fin, fout, len(free), float(f)))

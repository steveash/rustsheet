# T-1941 review (Fable 5.1): normalisation survey of the candidate sets: max|coef|, which coefficient is a power of two, values at interval ends.
import sys; sys.path.insert(0,'.')
from k1lib import load_cand
from k23lib import N3,D3,P2,Q2
import numpy as np, math
N1,D1=load_cand('../g_candidate_w4_AN2.txt')
def show(nm,N,D,a,b):
    print(nm)
    for lab,c in (('N',N),('D',D)):
        m=max(abs(v) for v in c); print('  %s deg %d max|c|=%.4g (2^%.3f) at idx %d; c0=%.6g (2^%.3f); top=%.4g (2^%.2f)'%(lab,len(c)-1,m,math.log2(m),[abs(v) for v in c].index(m),c[0],math.log2(abs(c[0])) if c[0] else float('nan'),c[-1],math.log2(abs(c[-1]))))
        print('    sum|c| at a,b: %.4g %.4g ; value at a,b: %.6g %.6g'%(sum(abs(v)*a**i for i,v in enumerate(c)),sum(abs(v)*b**i for i,v in enumerate(c)),sum(v*a**i for i,v in enumerate(c)),sum(v*b**i for i,v in enumerate(c))))
show('K1 AN2 [1,2)',N1,D1,1,2)
show('K3 c5 [0.7,1.5)',N3,D3,0.7,1.5)
show('K2 grid_a (2,3]',P2[::-1],Q2[::-1],2,3)

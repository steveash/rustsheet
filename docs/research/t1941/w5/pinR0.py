# How tightly does near-1 data pin the x=0 invariant R0 = N0/D0?  Move N0 by d and N1 by -d (S0 fixed).
import numpy as np
from sc import candidate, score
from metrics import XF, BF, XN, BN
N=candidate('N3'); D=candidate('D3')
for d in [0,1e-16,1e-15,1e-14,1e-13,1e-12,1e-11,1e-10,1e-9,1e-8,1e-7]:
    r=[]
    for sg in (1,-1):
        n=N.copy(); n[0]+=sg*d; n[1]-=sg*d; r.append((score(XN,BN,n,D),score(XF,BF,n,D)))
    print('dN0=%.0e  rel R0 change %.1e  near %.4f %.4f  far %.4f %.4f'%(d,d/abs(N[0]),r[0][0],r[1][0],r[0][1],r[1][1]))

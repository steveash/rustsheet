# compensated moves: change only stage sum S_j at x=1 (N_j += d, N_{j-1} -= d) and measure near-1 exact rate
import numpy as np
from sc import candidate
from metrics import XN, BN, score
N=candidate('N3'); D=candidate('D3'); base=score(XN,BN,N,D); print('c5 near-1 exact',base)
for ch in 'ND':
    for j in range(1,9):
        row=[]
        for k in [-4,-2,-1,1,2,4]:
            n=N.copy(); d=D.copy(); a=n if ch=='N' else d
            S=abs(sum(a[j:])); u=np.spacing(S)*k; a[j]+=u; a[j-1]-=u
            row.append(score(XN,BN,n,d))
        print('  only S%d of %s moved by -4,-2,-1,1,2,4 ulp:'%(j,ch),' '.join('%.4f'%v for v in row))

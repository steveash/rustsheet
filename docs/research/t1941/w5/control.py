# positive controls for the x=0 invariant test
import mpmath as mp, pickle, numpy as np
from taylor0 import taylor
from sc import candidate
from metrics import XN, BN, score
mp.mp.dps=50
P=pickle.load(open('parents.pkl','rb'))
p,q,E=P[(8,8,'0.7','1.5','rel')]; p=[mp.mpf(v) for v in p]; q=[mp.mpf(v) for v in q]
def mul(c,F):
    o=[mp.mpf(0)]*(len(c)+len(F)-1)
    for i,a in enumerate(c):
        for j,f in enumerate(F): o[i+j]+=a*f
    return o
s=mp.mpf(2)**-13
t0=taylor(p,q)
print('(1) parent', [mp.nstr(v,15) for v in t0])
for F in [[1,'-0.00505'],[1,'0.0123','-0.0007'],[1,'-0.2','0.03','-0.001']]:
    F=[mp.mpf(v) for v in F]
    n=[float(v*s) for v in mul(p,F)]; d=[float(v*s) for v in mul(q,F)]
    t=taylor(n,d); print('   F=%s rel dev'%[mp.nstr(v,4) for v in F],[mp.nstr(a/b-1,3) for a,b in zip(t,t0)])
# (2) do near-1 data pin the low coefficients? move N_j / D_j by k ulps of the stage sum S_j at x=1
N=candidate('N3'); D=candidate('D3'); base=score(XN,BN,N,D); print('(2) c5 near-1 exact',base)
for ch,arr in [('N',N),('D',D)]:
    for j in range(1,6):
        row=[]
        for k in [-2,-1,1,2]:
            a=arr.copy(); S=abs(sum(arr[j:])); a[j]+=k*np.spacing(S)
            row.append(score(XN,BN,a,D) if ch=='N' else score(XN,BN,N,a))
        print('   %s%d'%(ch,j),' '.join('%.4f'%v for v in row))

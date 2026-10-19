# Coordinator's doublet test: Excel = round(2^-13*P8*(1+a x), 2^-13*Q8*(1+a x)); exact score vs a.
# Parents: (8,8) relative minimax of lgamma/(x-1) on [0.7,1.5], and the (8,8) relative minimax of c5's function.
import mpmath as mp, pickle, numpy as np, sys
from sc import score, candidate
from metrics import XF, BF, XN, BN
from remez2 import remez2
mp.mp.dps=40
P=pickle.load(open('parents.pkl','rb')); p,q,E=P[(8,8,'0.7','1.5','rel')]
par={'mm':([mp.mpf(v) for v in p],[mp.mpf(v) for v in q])}
Nc=[mp.mpf(v) for v in candidate('N3')[::-1]]; Dc=[mp.mpf(v) for v in candidate('D3')[::-1]]
pc,qc,Ec,_=remez2(lambda x: mp.polyval(Nc,x)/mp.polyval(Dc,x),0.7,1.5,8,8,dps=40,grid=500); par['c5fit']=(pc,qc)
s=mp.mpf(2)**-13
X=np.r_[XF[:30000],XN[:15000]]; B=np.r_[BF[:30000],BN[:15000]]; nf=30000
def build(p,q,al):
    def mul(c): return np.array([float(s*((c[i] if i<len(c) else 0)+al*(c[i-1] if i>0 else 0))) for i in range(len(c)+1)])
    return mul(p),mul(q)
print('c5: far %.4f near %.4f'%(score(X[:nf],B[:nf],candidate('N3'),candidate('D3')),score(X[nf:],B[nf:],candidate('N3'),candidate('D3'))))
for name,(p,q) in par.items():
    best=[]
    for al in np.r_[np.linspace(-0.03,0.03,6001),[-1/353.4234,1/353.4234]]:
        n,d=build(p,q,mp.mpf(al)); best.append((score(X[:nf],B[:nf],n,d),score(X[nf:],B[nf:],n,d),al))
    b=np.array(best); i=np.argmax(b[:,0]+b[:,1])
    print('%s: best far %.4f near %.4f at a=%.5f ; median far %.4f near %.4f ; a=0: far %.4f near %.4f'%(name,b[i,0],b[i,1],b[i,2],np.median(b[:,0]),np.median(b[:,1]),b[3000,0],b[3000,1]))

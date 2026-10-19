# Positive control: synthetic Excel = round(2^-13 * P8*(1+a0 x), 2^-13 * Q8*(1+a0 x)); score vs alpha offset.
import mpmath as mp, pickle, numpy as np
from sc import res, score
from metrics import XF, XN
mp.mp.dps=40
P=pickle.load(open('parents.pkl','rb')); p,q,E=P[(8,8,'0.7','1.5','rel')]
p=[mp.mpf(v) for v in p]; q=[mp.mpf(v) for v in q]; s=mp.mpf(2)**-13
def build(p,q,F):
    def mul(c):
        o=[mp.mpf(0)]*(len(c)+len(F)-1)
        for i,a in enumerate(c):
            for j,f in enumerate(F): o[i+j]+=a*f
        return np.array([float(v*s) for v in o])
    return mul(p),mul(q)
a0=mp.mpf('-0.0050512345678901234')
N0,D0=build(p,q,[1,a0])
X=np.r_[XF[:20000],XN[:10000]]; B=res(X,N0,D0)
print('self',score(X,B,N0,D0))
for rel in [1e-4,1e-6,1e-8,1e-10,1e-12,1e-13,1e-14,1e-15,1e-16]:
    sc_=[score(X,B,*build(p,q,[1,a0*(1+mp.mpf(rel)*k)])) for k in (-1,1)]
    print('alpha rel offset %.0e: %.4f %.4f'%(rel,*sc_))

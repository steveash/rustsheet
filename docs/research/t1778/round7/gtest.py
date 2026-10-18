import sys
from glob_ import *
ch=sys.argv[1]; y=yN if ch=='N' else yD
g=2.0**-48
Ws=W; rels=rel*Ws; s=(x-15)/10
N0=np.vstack([basis(eN1),basis(eD1)]).T*Ws[:,None]
Q,_=np.linalg.qr(N0); yr=rels-Q@(Q.T@rels); s2=(yr@yr)/len(yr)
def sc(b):
    w=y+b; p,e=two_prod_err(w,x); r3=-e/p
    A=np.vstack([r3,r3*s,r3*s*s]).T*Ws[:,None]; A=A-Q@(Q.T@A)
    co,*_=np.linalg.lstsq(A,yr,rcond=None); f=A@co
    return f@f/s2
for h in sys.argv[2:]:
    b0=float.fromhex(h)
    print(h, ' '.join('%+d:%.1f'%(k,sc(b0+k*g)) for k in range(-3,4)))

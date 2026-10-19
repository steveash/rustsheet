# Taylor coefficients of R = N/D at x=0 (invariant under any common factor F)
import mpmath as mp, pickle, numpy as np
from sc import candidate
mp.mp.dps=50
def taylor(p,q,k=4):
    # series division p/q
    p=[mp.mpf(v) for v in p]+[mp.mpf(0)]*k; q=[mp.mpf(v) for v in q]+[mp.mpf(0)]*k
    r=[]
    for i in range(k):
        r.append((p[i]-sum(r[j]*q[i-j] for j in range(i)))/q[0])
    return r
def show(tag,p,q): print('%-28s'%tag,' '.join(mp.nstr(v,16) for v in taylor(p,q)))
if __name__=="__main__": show('c5 (N3/D3)',candidate('N3'),candidate('D3'))
if __name__=="__main__": show('W1 C1 (N2/D2)',candidate('N2'),candidate('D2'))
if __name__=="__main__": P=pickle.load(open('parents.pkl','rb'))
if __name__=="__main__":
  for k,(p,q,E) in P.items(): show(str(k[:4]),p,q)

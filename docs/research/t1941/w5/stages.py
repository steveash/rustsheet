import numpy as np
from sc import candidate
def stages1(c):
    """x87 Horner partial sums at x=1: S[j] after adding c[j] (j from top down); returns dict j->S."""
    s=c[-1]; out={len(c)-1:s}
    for j in range(len(c)-2,-1,-1):
        s=float(np.longdouble(s)+np.longdouble(c[j])); out[j]=s
    return out
if __name__=='__main__':
    for tn,td in [('N3','D3'),('N2','D2')]:
        N=candidate(tn);D=candidate(td); sn=stages1(N); sd=stages1(D)
        print(tn, ' '.join('%d:%s'%(j,sn[j].hex()) for j in sorted(sn)))
        print(td, ' '.join('%d:%s'%(j,sd[j].hex()) for j in sorted(sd)))

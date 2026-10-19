# Binade crossings of every c5_f99t Horner intermediate (products P_j = S_{j+1}*x, sums S_j, A, res) on [0.7,1.5]
import numpy as np, math
from sc import candidate
N=candidate('N3'); D=candidate('D3')
x=np.linspace(0.7,1.5,800001)
def chain(c,tag):
    out={}; s=np.full_like(x,c[-1])
    for j in range(len(c)-2,-1,-1):
        p=s*x; out['%s.P%d'%(tag,j)]=p; s=p+c[j]; out['%s.S%d'%(tag,j)]=s
    return out,s
on,Nv=chain(N,'N'); od,Dv=chain(D,'D'); on.update(od)
on['A']=Nv/Dv; on['res']=(x-1)*on['A']
for k,v in on.items():
    e=np.floor(np.log2(np.abs(v)))
    ch=np.where(np.diff(e)!=0)[0]
    if len(ch): print('%-6s'%k, ' '.join('%.4f(2^%d->%d)'%(x[i],e[i],e[i+1]) for i in ch if abs(x[i]-1)>1e-4))

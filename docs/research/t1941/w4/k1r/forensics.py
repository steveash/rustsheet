import numpy as np, sys
sys.path.insert(0,'../wt/docs/research/t1941/w6'); from synth import read_chain
LD=np.longdouble
def st(v): return np.asarray(v,dtype=LD).astype(np.float64).astype(LD)
d=np.load('k1R10.npz'); x,t=d['x'][::3],d['t'][::3]; N,D=read_chain(sys.argv[1] if len(sys.argv)>1 else 'plain1.txt'); N=np.array(N); D=np.array(D); Y=x.astype(LD)
def chain(C):
    s=np.full(len(x),LD(C[-1]))
    for j in range(6,-1,-1): s=st(st(s*Y)+LD(C[j]))
    return s.astype(np.float64)
Nv=chain(N); Dv=chain(D); best=np.full(len(x),99)
for k in range(-3,4):
    for m in range(-3,4):
        g=((Nv+k*np.spacing(np.abs(Nv))).astype(LD)/(Dv+m*np.spacing(np.abs(Dv))).astype(LD)).astype(np.float64); c=abs(k)+abs(m); ok=(g==t)&(c<best); best[ok]=c
for lo,hi in ((1.0,1.4),(1.4,1.8),(1.84,1.97),(1.97,2.0)):
    b=best[(x>=lo)&(x<hi)]; print('%.2f-%.2f exact %.3f 1 step %.3f 2 %.3f 3+ %.3f unfixable %.3f'%(lo,hi,np.mean(b==0),np.mean(b==1),np.mean(b==2),np.mean((b>=3)&(b<99)),np.mean(b==99)))

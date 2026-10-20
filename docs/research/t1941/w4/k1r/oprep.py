import k1lib as K, numpy as np, pickle, sys, os
LD=np.longdouble
def st(v): return v.astype(np.float64).astype(LD)
def G(N,D,y,keep=None):
    Y=y.astype(LD); out=[]
    for ch,C in (('N',N),('D',D)):
        s=np.full(len(y),LD(C[-1]))
        for k in range(7):
            p=s*Y
            if keep!=(ch,'mul',k): p=st(p)
            s=p+LD(C[6-k])
            if keep!=(ch,'add',k): s=st(s)
        out.append(s)
    return (out[0]/out[1]).astype(np.float64)
def load(f):
    d=pickle.load(open(f,'rb')); return np.array(d['N']),np.array(d['D'])
src=sys.argv[1]; runs=sys.argv[2].split('+'); ylo=float(sys.argv[3]); yhi=float(sys.argv[4]); stp=int(sys.argv[5])
N,D=(K.N0,K.D0) if src=='AN2' else load(src)
R=[r for r in K.runs(runs) if ylo<=r[1][0]<yhi]
Y=np.concatenate([r[1][::stp] for r in R]); T=np.concatenate([r[2][::stp] for r in R])
b=(G(N,D,Y)==T); print('%s on %s: base %.4f (%d pts)'%(src,sys.argv[2][:40],b.mean(),len(Y)))
res=[]
for ch in ('N','D'):
    for kind in ('mul','add'):
        for k in range(7):
            m=(G(N,D,Y,(ch,kind,k))==T); res.append((int((m&~b).sum()-(b&~m).sum()),int((m&~b).sum()),int((b&~m).sum()),'%s.%s%d'%(ch,kind,k+1)))
res.sort(reverse=True)
print(' top net repairs (net, repaired, broken, op):',' | '.join('%+d %d %d %s'%t for t in res[:6]))
print(' worst:',' | '.join('%+d %s'%(t[0],t[3]) for t in res[-3:]))

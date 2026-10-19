# T-1941 W6: for a candidate rational R = N/D (raw-x Horner, degrees n,d, fitted on [lo,hi]), scale both chains by
# lambda (N and D scaled together keep R; Excel's normalisation is unknown) and count observed sweep peaks matched by
# the predicted stage lines (product sawtooth P'h/ulp(P); sum sawtooth S'h/ulp(S); triangle S'h/(2ulp(S))).
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from rfit import fit
d=np.loadtxt(sys.argv[1]); d=d[d[:,2]>0.3]; xs=np.unique(d[:,0])
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def stages(c,x):
    # c ascending floats; returns list of (value, derivative) arrays for products and sums
    n=len(c)-1; s=np.full_like(x,c[n]); sd=np.zeros_like(x); out=[]
    for j in range(n-1,-1,-1):
        P=s*x; Pd=sd*x+s; out.append(('P%d'%j,P,Pd)); s=P+c[j]; sd=Pd; out.append(('S%d'%j,s,sd))
    return out
def ulp(v): return np.spacing(np.abs(v))
def lines(c,x):
    L=[]
    for nm,v,vd in stages(c,x):
        F=vd*2.0**-52/ulp(v); L.append((nm,alias(F)))
        if nm[0]=='S': L.append((nm+'t',alias(F/2)))
    return L
xi=np.searchsorted(xs,d[:,0]); tol=0.0012
def score(N,D,lam):
    tot=np.zeros(len(d),bool); names={}
    for ch,c in (('N',N),('D',D)):
        for nm,f in lines([v*lam for v in c],xs):
            hit=np.abs(f[xi]-d[:,1])<tol; tot|=hit; names[ch+nm]=hit.sum()
    return tot.sum(),names
if __name__=='__main__':
    n,dd,lo,hi=sys.argv[2].split(','); a,b,e=fit(int(n),int(dd),mp.mpf(lo),mp.mpf(hi))
    N=[float(v) for v in a]; D=[float(v) for v in b]
    res=[]
    for lam in np.exp2(np.arange(0,1,1/float(sys.argv[3]))):
        s,nm=score(N,D,lam); res.append((s,lam,nm))
    res.sort(key=lambda r:-r[0])
    print('peaks %d; chance ~%.0f'%(len(d),0))
    for s,lam,nm in res[:5]: print('%d lam %.5f %s'%(s,lam,sorted(nm.items(),key=lambda kv:-kv[1])[:8]))
    print('median', np.median([r[0] for r in res]))

# T-1941 W6: long-run line spectra. For each long run (consecutive doubles), the R residual (Excel R midpoint, or a
# simulated chain) minus the true R, Hann-windowed FFT, top-K peaks with quadratic interpolation.
# Usage: longspec.py synth CHAIN OUT.npz [n]   (synthetic runs at the L101..L124 x0)
#        longspec.py excel RFILE OUT.npz        (rows of kmyst-gam-L1xx from an R file)
import sys, numpy as np, mpmath as mp
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner; from synth import read_chain
mp.mp.dps=40
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
X0=[1.5047+0.0203913*i+0.00071*((0.618034*i)%1.0) for i in range(24)]
def resid(x,Rlo,Rhi):
    x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,j)/mp.factorial(j) for j in range(4)]
    d=(x-x[0]); dm=[mp.mpf(float(v)) for v in d]   # exact: x - x0 is exact for consecutive doubles
    tr=[g[0]+t*(g[1]+t*(g[2]+t*g[3])) for t in dm]
    hi=np.array([float(t) for t in tr]); lo=np.array([float(t-mp.mpf(float(t))) for t in tr])
    return (((Rlo+Rhi)/2-hi)-lo)/np.spacing(Rlo)
def peaks(e,K=40):
    n=len(e); w=np.hanning(n); F=np.abs(np.fft.rfft((e-e.mean())*w))/w.sum()*2
    med=np.median(F[20:]); idx=[j for j in range(10,len(F)-1) if F[j]>F[j-1] and F[j]>=F[j+1] and F[j]>6*med]
    idx.sort(key=lambda j:-F[j]); out=[]
    for j in idx[:K]:
        a,b,c=np.log(F[j-1]),np.log(F[j]),np.log(F[j+1]); dlt=0.5*(a-c)/(a-2*b+c); out.append(((j+dlt)/n,F[j]))
    return out
def extractR(x,B):
    t=x-1.0; m=x-2.0; R0=B/(t*m); H=[]
    for k in range(-4,5):
        Rk=R0.copy()
        for _ in range(abs(k)): Rk=np.nextafter(Rk,np.inf if k>0 else -np.inf)
        H.append(np.where(st(L(t)*L(st(L(m)*L(Rk))))==B,Rk,np.nan))
    H=np.array(H); return np.nanmin(H,0),np.nanmax(H,0)
if __name__=='__main__':
    res={}
    if sys.argv[1]=='synth':
        N,D=read_chain(sys.argv[2]); n=int(sys.argv[4]) if len(sys.argv)>4 else 65536
        for i,x0 in enumerate(X0):
            x=np.empty(n); x[0]=x0
            for k in range(1,n): x[k]=np.nextafter(x[k-1],3.0)
            R=st(L(horner(N[::-1],x))/L(horner(D[::-1],x))); B=st(L(x-1.0)*L(st(L(x-2.0)*L(R))))
            lo,hi=extractR(x,B); res['x%d'%i]=np.array([x0]); res['p%d'%i]=np.array(peaks(resid(x,lo,hi)))
            print(i,'%.5f'%x0,flush=True)
    else:
        import collections; rows=collections.defaultdict(list)
        for l in open(sys.argv[2]):
            p=l.split()
            if p[5].startswith('kmyst-gam-L1'): rows[p[5]].append((float.fromhex(p[0]),float.fromhex(p[2]),float.fromhex(p[3])))
        for i,(k,v) in enumerate(sorted(rows.items())):
            v=np.array(sorted(v)); res['x%d'%i]=np.array([v[0,0]]); res['p%d'%i]=np.array(peaks(resid(v[:,0],v[:,1],v[:,2]))); print(k,len(v),flush=True)
    np.savez(sys.argv[3],**res)

# T-1941 W6: exact-simulation spectral objective. Precomputes, for the sweep segments of an R file (512 consecutive
# doubles each), the true R and Excel's residual |FFT|; a candidate chain is simulated (raw-x Horner, every op stored,
# R = fl(N/D)) and scored by the mean per-segment correlation of |FFT| (bins 3..256).
import sys, numpy as np, mpmath as mp, collections, os, pickle
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
from lib6 import st, L, horner
mp.mp.dps=30
Rf=lambda z: mp.loggamma(z)/((z-1)*(z-2))
def build(rpath,cache,prefix='kmyst-gam-q40',maxseg=400):
    if os.path.exists(cache): return pickle.load(open(cache,'rb'))
    seg=collections.defaultdict(list)
    for l in open(rpath):
        p=l.split()
        if not p[5].startswith(prefix): continue
        x=float.fromhex(p[0]); seg[(p[5],round(x,4))].append((x,float.fromhex(p[2]),float.fromhex(p[3])))
    X=[];T=[];E=[]
    for k,v in sorted(seg.items(),key=lambda kv:kv[1][0][0]):
        v=np.array(v)
        if len(v)<512: continue
        v=v[:512]; x=v[:,0]; x0=mp.mpf(x[0]); g=[mp.diff(Rf,x0,j)/mp.factorial(j) for j in range(3)]
        tr=[g[0]+(mp.mpf(xi)-x0)*(g[1]+(mp.mpf(xi)-x0)*g[2]) for xi in x]
        hi=np.array([float(t) for t in tr]); lo=np.array([float(t-mp.mpf(float(t))) for t in tr])
        Rm=(v[:,1]+v[:,2])/2; u=np.spacing(v[:,1]); e=((Rm-hi)-lo)/u
        F=np.abs(np.fft.rfft(e-e.mean()))[3:257]
        X.append(x); T.append((hi,lo,u)); E.append(F)
        if len(X)>=maxseg: break
    D=(np.array(X),T,np.array(E)); pickle.dump(D,open(cache,'wb')); return D
class SimObj:
    def __init__(self,data): self.X,self.T,self.E=data; self.Xa=self.X.reshape(-1); self.Ez=(self.E-self.E.mean(1,keepdims=True))/self.E.std(1,keepdims=True)
    def __call__(self,N,D,per=False):
        R=st(L(horner(list(N[::-1]),self.Xa))/L(horner(list(D[::-1]),self.Xa))).reshape(self.X.shape)
        out=[]
        for i,(hi,lo,u) in enumerate(self.T):
            e=((R[i]-hi)-lo)/u; F=np.abs(np.fft.rfft(e-e.mean()))[3:257]; F=(F-F.mean())/F.std()
            out.append(np.mean(F*self.Ez[i]))
        return np.array(out) if per else float(np.mean(out))

# T-1941 W6: pole-parametrised chain model for R on (1.5,2). D(x) = prod(1 - x/p_i) (D0 = 1; a power-of-two D0 gives
# the same frequencies), N = degree-n LS fit of R*D (relative). Stage lines from raw-x Horner of N and D; objective =
# amplitude-weighted soft match of sweep peaks.
import numpy as np, mpmath as mp
mp.mp.dps=40
XF=np.cos(np.pi*(np.arange(80)+0.5)/80)*0.25+1.75
RT=np.array([float(mp.loggamma(mp.mpf(x))/((mp.mpf(x)-1)*(mp.mpf(x)-2))) for x in XF])
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def dpoly(poles):
    c=np.array([1.0])
    for p in poles: c=np.convolve(c,[1.0,-1.0/p])   # ascending: (1 - x/p)
    return c
def npoly(D,n):
    y=RT*np.polyval(D[::-1],XF); V=np.vander(XF,n+1,increasing=True)/y[:,None]
    c,*_=np.linalg.lstsq(V,np.ones_like(y),rcond=None); return c
def stage_lines(c,x):
    n_=len(c)-1; s=np.full_like(x,c[n_]); sd=np.zeros_like(x); L=[]
    for j in range(n_-1,-1,-1):
        P=s*x; Pd=sd*x+s; L.append(alias(Pd*2.0**-52/np.spacing(np.abs(P)))); s=P+c[j]; sd=Pd
        F=sd*2.0**-52/np.spacing(np.abs(s)); L.append(alias(F)); L.append(alias(F/2))
    return L
class Obj:
    def __init__(self,peaks,n,amin=0.35):
        d=np.loadtxt(peaks); d=d[d[:,2]>amin]; self.xs=np.unique(d[:,0]); self.xi=np.searchsorted(self.xs,d[:,0]); self.f=d[:,1]; self.a=d[:,2]; self.n=n
    def chain(self,poles):
        D=dpoly(poles); N=npoly(D,self.n); return N,D
    def __call__(self,poles,sig=0.002,hard=False):
        N,D=self.chain(poles); best=np.zeros(len(self.f))
        for Lf in stage_lines(N,self.xs)+stage_lines(D,self.xs):
            dv=np.abs(Lf[self.xi]-self.f)
            best=np.maximum(best,(dv<0.0012).astype(float) if hard else np.exp(-(dv/sig)**2))
        return (self.a*best).sum()/self.a.sum()
    def funerr(self,poles):
        N,D=self.chain(poles); return np.max(np.abs(np.polyval(N[::-1],XF)/np.polyval(D[::-1],XF)/RT-1))

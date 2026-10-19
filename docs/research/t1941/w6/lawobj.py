# T-1941 W6: line-match objective with the full law set (stagelaws + quotient): fraction of peaks within tol of any
# law of chain (N,D). Used to test identifiability (truth vs perturbations / false optima).
import numpy as np, sys
sys.path.insert(0,sys.path[0]); from stagelaws import stage_F, quotient_F
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
class LawObj:
    def __init__(self,peaks,amin=0.3,tol=4e-4):
        d=np.loadtxt(peaks); d=d[d[:,2]>amin]; self.xs=np.unique(d[:,0]); self.xi=np.searchsorted(self.xs,d[:,0]); self.f=d[:,1]; self.a=d[:,2]; self.tol=tol
    def __call__(self,N,D):
        hit=np.zeros(len(self.f),bool)
        fams=[F for c in (N,D) for (F,v,vd) in stage_F(np.asarray(c,float),self.xs).values()]+[F for (F,v,vd) in quotient_F(np.asarray(N,float),np.asarray(D,float),self.xs).values()]
        for F in fams: hit|=np.abs(alias(F)[self.xi]-self.f)<self.tol
        return hit.mean()

# Control for globalfit: permute which run gets which x (destroys any smooth-in-x structure) and repeat the search;
# report the distribution of the best summed rate so the real gain can be judged against overfitting.
import numpy as np, time, sys
d=np.load('profiles.npy',allow_pickle=True).item()   # written by globalfit.py
RUNS=["r701","r112","p401","r702","r610","r901","r703","r113","v401a","r704","r611","r705","v407a"]
x0=np.array([d['xs'][c] for c in RUNS]); PN=np.array([d['prof'][c][0] for c in RUNS]); PD=np.array([d['prof'][c][1] for c in RUNS])
def gsearch(P,x,basis,ranges):
    B=np.array([b(x) for b in basis]); nr=len(x)
    grids=[np.arange(lo,hi+1,st) for lo,hi,st in ranges]
    G=np.meshgrid(*grids[:-1],indexing='ij'); flat=np.stack([g.ravel() for g in G],1); last=grids[-1]; best=-1
    for row in flat:
        pred=(row@B[:-1])[None,:]+last[:,None]*B[-1][None,:]
        idx=np.rint(np.mod(pred,2.0)*8).astype(int)%16
        sc=P[np.arange(nr)[None,:],idx].sum(1); best=max(best,float(sc.max()))
    return best
bN=[lambda x:np.ones_like(x),lambda x:x/8,lambda x:x**2/128]; rN=[(-3,3,1),(-64,64,1),(-2000,2000,1)]
bD=[lambda x:0.5*np.ones_like(x),lambda x:x/16,lambda x:x**2/512]; rD=[(-3,3,1),(-64,64,1),(-2000,2000,1)]
rng=np.random.default_rng(1)
real=(gsearch(PN,x0,bN,rN),gsearch(PD,x0,bD,rD))
print(f"real: N {real[0]:.3f} (+{real[0]-PN[:,0].sum():.3f} of +{PN.max(1).sum()-PN[:,0].sum():.3f}), D {real[1]:.3f} (+{real[1]-PD[:,0].sum():.3f} of +{PD.max(1).sum()-PD[:,0].sum():.3f})",flush=True)
ctlN=[];ctlD=[]
for i in range(12):
    perm=rng.permutation(len(x0)); xp=x0[perm]
    ctlN.append(gsearch(PN,xp,bN,rN)-PN[:,0].sum()); ctlD.append(gsearch(PD,xp,bD,rD)-PD[:,0].sum())
    print(f"perm {i}: N +{ctlN[-1]:.3f}  D +{ctlD[-1]:.3f}",flush=True)
print(f"controls: N gain mean {np.mean(ctlN):.3f} max {np.max(ctlN):.3f}; D gain mean {np.mean(ctlD):.3f} max {np.max(ctlD):.3f}")

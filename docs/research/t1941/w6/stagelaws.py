# T-1941 W6: unaliased stage-line laws of a raw-x Horner chain, and oracle assignment of tracks to them.
import numpy as np
def stage_F(c,x):
    """dict name -> (F unaliased, value, derivative) for products P_j, sums S_j (sawtooth) and S_j/2 (triangle)."""
    n_=len(c)-1; s=np.full_like(x,c[n_]); sd=np.zeros_like(x); out={}
    for j in range(n_-1,-1,-1):
        P=s*x; Pd=sd*x+s; out['P%d'%j]=(Pd*2.0**-52/np.spacing(np.abs(P)),P,Pd)
        s=P+c[j]; sd=Pd; F=sd*2.0**-52/np.spacing(np.abs(s)); out['S%d'%j]=(F,s,sd); out['T%d'%j]=(F/2,s,sd)
    return out
def assign(track,chains,maxrms=0.0015):
    x=np.array(track['x']); Fo=np.array(track['F']); best=None
    for ch,c in chains.items():
        for nm,(F,v,vd) in stage_F(c,x).items():
            for sg in (1,-1):
                dlt=sg*Fo-F; n=np.round(np.median(dlt)); r=np.sqrt(np.mean((dlt-n)**2))
                if best is None or r<best[0]: best=(r,ch+'.'+nm,sg,int(n))
    return best if best[0]<maxrms else (best[0],None,0,0)
if __name__=='__main__':
    import sys, json
    sys.path.insert(0,sys.path[0]); from synth import read_chain
    N,D=read_chain(sys.argv[1]); T=json.load(open(sys.argv[2]))
    for i,t in enumerate(T):
        r,nm,sg,n=assign(t,{'N':np.array(N),'D':np.array(D)})
        print('T%-2d n=%3d  best %-6s sign %+d int %+d rms %.5f'%(i,len(t['x']),nm,sg,n,r))

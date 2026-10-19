# T-1941 W6: unaliased stage-line laws of a raw-x Horner chain, and oracle assignment of tracks to them.
import numpy as np
def stage_F(c,x):
    """dict name -> (F unaliased, value, derivative) for products P_j, sums S_j (sawtooth) and S_j/2 (triangle)."""
    n_=len(c)-1; s=np.full_like(x,c[n_]); sd=np.zeros_like(x); out={}
    for j in range(n_-1,-1,-1):
        P=s*x; Pd=sd*x+s; FP=Pd*2.0**-52/np.spacing(np.abs(P)); out['P%d'%j]=(FP,P,Pd)
        if j<n_-1:   # W4 sideband law: multiplicand M = s (stored sum S_{j+1}); F_n = FP - n*F_M
            FM=sd*2.0**-52/np.spacing(np.abs(s))
            for nn in (1,2,3,-1): out['P%dn%d'%(j,nn)]=(FP-nn*FM,P,Pd)
        s=P+c[j]; sd=Pd; F=sd*2.0**-52/np.spacing(np.abs(s)); out['S%d'%j]=(F,s,sd); out['T%d'%j]=(F/2,s,sd)
        for nn in (1,2,-1,-2): out['S%dm%d'%(j,nn)]=(F-nn*FP,s,sd)   # sum phase-modulated by the stored product's saw
    return out
def quotient_F(N,D,x):
    """Rounding line of R = fl(N/D) and its sidebands from the stored N (=S0 of N) and D (=S0 of D)."""
    a=stage_F(N,x); b=stage_F(D,x); Nv=a['S0'][1]; Nd=a['S0'][2]; Dv=b['S0'][1]; Dd=b['S0'][2]
    R=Nv/Dv; Rd=(Nd*Dv-Nv*Dd)/Dv**2; FR=Rd*2.0**-52/np.spacing(np.abs(R)); FN=a['S0'][0]; FD=b['S0'][0]; out={}
    for i in (-2,-1,0,1,2):
        for k in (-2,-1,0,1,2): out['Q%+d%+d'%(i,k)]=(FR-i*FN-k*FD,R,Rd)
    return out

def assign(track,chains,maxrms=0.0015):
    x=np.array(track['x']); Fo=np.array(track['F']); best=None
    items=[(ch+'.'+nm,F) for ch,c in chains.items() for nm,(F,v,vd) in stage_F(c,x).items()]
    if 'N' in chains and 'D' in chains: items+=[('R.'+nm,F) for nm,(F,v,vd) in quotient_F(chains['N'],chains['D'],x).items()]
    for full,F in items:
        if True:
            for sg in (1,-1):
                dlt=sg*Fo-F; n=np.round(np.median(dlt)); r=np.sqrt(np.mean((dlt-n)**2))
                if best is None or r<best[0]: best=(r,full,sg,int(n))
    return best if best[0]<maxrms else (best[0],None,0,0)
if __name__=='__main__':
    import sys, json
    sys.path.insert(0,sys.path[0]); from synth import read_chain
    N,D=read_chain(sys.argv[1]); T=json.load(open(sys.argv[2]))
    for i,t in enumerate(T):
        r,nm,sg,n=assign(t,{'N':np.array(N),'D':np.array(D)})
        print('T%-2d n=%3d  best %-6s sign %+d int %+d rms %.5f'%(i,len(t['x']),nm,sg,n,r))

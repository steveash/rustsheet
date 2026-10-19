# T-1941 W6: track-level EM from a good prior chain. Each grown track (precise, 7-24 runs) is compared with every law
# prediction of the current chain along its whole x-range (best sign/int); it is accepted if the rms is below tol and
# the next-best law is > 3x worse. Accepted tracks join the seeds; re-solve; tol shrinks. Usage as module.
import numpy as np
from fastenum import law_key
from stagelaws import stage_F
def best_law(N,D,x,F,n,laws=('T','Pn1')):
    out=[]
    for ch,c in (('N',N),('D',D)):
        sf=stage_F(c,x); deg=len(c)-1   # each chain's own degree (unequal (n,d) allowed)
        for law in laws:
            for j in range(deg-1 if law[0]=='P' else deg):
                pr=sf[law_key(law,j)][0]
                for sg in (1,-1):
                    k=np.round(np.median(pr-sg*F)); r=np.sqrt(np.mean((pr-(sg*F+k))**2)); out.append((r,(ch,law,j,sg,int(k))))
    out.sort(key=lambda t:t[0]); return out
def run(fe,tracks,seed_h,seed_t,geo,rounds=6,tol=3e-3,truth=None,zc=None):
    if zc is None: zc,_=fe.solve(list(seed_h),list(seed_t))
    for rnd in range(rounds):
        N,D=fe.chain(zc); hyps=list(seed_h); trs=list(seed_t); acc=[]
        for ti,(x,F) in enumerate(tracks):
            bl=best_law(N,D,x,F,fe.n)
            if bl[0][0]<tol and bl[1][0]>3*bl[0][0]:
                h=bl[0][1]; hyps.append(h+(geo,)); trs.append((x,F)); acc.append((ti,'%s.%s%d'%h[:3],bl[0][0]))
        zc,res=fe.solve(hyps,trs)
        msg='round %d tol %.0e: %d tracks %s; max res %.1e'%(rnd,tol,len(acc),' '.join('%d:%s'%(a[0],a[1]) for a in acc),max(res))
        if truth is not None:
            Nn,Dd=fe.chain(zc); xg=np.linspace(1.5,2,11); Nt,Dt=truth
            e=max(np.max(np.abs(stage_F(Nn,xg)['S%d'%j][1]/stage_F(Nt,xg)['S%d'%j][1]-1)) for j in range(0,6))
            msg+='  max rel err S0-S5(N) %.1e'%e
        print(msg,flush=True)
        tol=max(tol/3,3e-6)
    return zc

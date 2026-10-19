# T-1941 W6: EM over per-run peaks. From a seed assignment (tracks -> laws), solve the chain (FastEnum), then for every
# long run and every law line of the current chain (T/Pn1, both chains) find the nearest observed peak; accept matches
# within tol that are unique (no other law within 3*tol), add them as one-point rows, re-solve; shrink tol.
# Usage as module: run(fe, peaks_npz, seed_hyps, seed_tracks, rounds, tol0)
import numpy as np
from fastenum import law_key
from stagelaws import stage_F
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def run(fe,z,seed_h,seed_t,rounds=6,tol0=2e-3,laws=('T','Pn1'),truth=None,verbose=True,adaptive=False,tolmin=3e-6):
    nr=len([k for k in z.files if k.startswith('x')])
    X=[z['x%d'%i][0]+32768*2.0**-52 for i in range(nr)]; P=[z['p%d'%i][:,0] for i in range(nr)]
    hyps=list(seed_h); trs=list(seed_t); zc,_=fe.solve(hyps,trs); tol=tol0
    for rnd in range(rounds):
        N,D=fe.chain(zc); acc=[]
        for i in range(nr):
            x=np.array([X[i]]); pred=[]
            for ch,c in (('N',N),('D',D)):
                sf=stage_F(c,x)
                for law in laws:
                    for j in range(fe.n-1 if law[0]=='P' else fe.n):
                        pred.append(((ch,law,j),sf[law_key(law,j)][0][0]))
            pa=np.array([alias(p) for _,p in pred])
            for (key,F),fa in zip(pred,pa):
                d=np.abs(P[i]-fa); jm=np.argmin(d)
                if d[jm]<tol and np.sum(np.abs(pa-P[i][jm])<3*tol)==1:
                    fo=P[i][jm]
                    # sign/int so that sg*fo+kk is closest to the unaliased prediction F
                    best=min(((abs(sg*fo+np.round(F-sg*fo)-F),sg,int(np.round(F-sg*fo))) for sg in (1,-1)))
                    acc.append(((key[0],key[1],key[2],best[1],best[2],0),(x,np.array([fo]))))
        allh=hyps+[a[0] for a in acc]; allt=trs+[a[1] for a in acc]
        # exponents in rows come from c0 (+eoff); recentre: use the seed eoff for all accepted rows of that chain
        geo=hyps[0][5]   # one global exponent offset (N = R*D scale together)
        allh=[h if k<len(hyps) else (h[0],h[1],h[2],h[3],h[4],geo) for k,h in enumerate(allh)]
        # trimmed refits: drop accepted matches with the largest residuals until all are below 5*tol
        keep=list(range(len(allh)))
        for it in range(12):
            try: zc2,res=fe.solve([allh[k] for k in keep],[allt[k] for k in keep])
            except Exception: break
            thr_r=max(3*np.median(res),2e-6) if adaptive else 5*tol
            bad=[keep[i] for i,r in enumerate(res) if r>thr_r and keep[i]>=len(hyps)]
            if not bad: break
            worst=sorted(bad,key=lambda k: -res[keep.index(k)])[:max(1,len(bad)//3)]
            keep=[k for k in keep if k not in worst]
        zc=zc2
        good=keep
        msg='round %d tol %.0e: %d matches, %d kept, max res %.1e'%(rnd,tol,len(acc),len(good)-len(hyps),max(res))
        if truth is not None:
            from stagelaws import stage_F as SF
            Nn,Dd=fe.chain(zc); xg=np.linspace(1.5,2,11); Nt,Dt=truth
            e=max(np.max(np.abs(SF(Nn,xg)['S%d'%j][1]/SF(Nt,xg)['S%d'%j][1]-1)) for j in range(0,6))
            msg+='  max rel err S0-S5(N) %.1e'%e
        if verbose: print(msg,flush=True)
        if not adaptive or len(good)-len(hyps)>=getattr(run,'_lastkept',0): tol=max(tol/2,tolmin)
        run._lastkept=len(good)-len(hyps)
    return zc

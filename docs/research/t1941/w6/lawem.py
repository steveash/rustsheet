# T-1941 W6: law-track EM. For the current chain, every law line (T/Pn1, both chains) is matched to the nearest peak
# in each long run (within tol); the residuals (observed - predicted, unaliased) must follow a smooth quadratic in x
# (the chain's own prediction error is smooth); a law is accepted if >= minrun runs are inliers within itol of that
# quadratic. Accepted laws become multi-point tracks; re-solve; iterate. Robust where per-peak EM is not.
import numpy as np
from fastenum import law_key
from stagelaws import stage_F
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def run(fe,z,seed_h,seed_t,rounds=6,tol=0.02,itol=3e-5,minrun=8,laws=('T','Pn1'),truth=None,verbose=True):
    nr=len([k for k in z.files if k.startswith('x')])
    X=np.array([z['x%d'%i][0]+32768*2.0**-52 for i in range(nr)]); P=[z['p%d'%i][:,0] for i in range(nr)]
    geo=seed_h[0][5]; zc,_=fe.solve(list(seed_h),list(seed_t)); acc_prev=set()
    for rnd in range(rounds):
        N,D=fe.chain(zc); hyps=list(seed_h); trs=list(seed_t); acc=[]
        for ch,c in (('N',N),('D',D)):
            sf=stage_F(c,X)
            for law in laws:
                for j in range(fe.n-1 if law[0]=='P' else fe.n):
                    F=sf[law_key(law,j)][0]
                    # for both alias signs: per run nearest peak to the aliased prediction
                    xs=[];res=[];fo=[]
                    for i in range(nr):
                        fa=alias(F[i]); d=np.abs(P[i]-fa); jm=np.argmin(d)
                        if d[jm]<tol:
                            o=P[i][jm]; sg=1 if (F[i]%1.0)<0.5 else -1   # unaliased: F = k + sg*o
                            k=np.round(F[i]-sg*o); xs.append(X[i]); fo.append((sg,int(k),o)); res.append(sg*o+k-F[i])
                    if len(xs)<minrun: continue
                    xs=np.array(xs); res=np.array(res)
                    # robust quadratic: iterate fit / inlier selection
                    inl=np.ones(len(xs),bool)
                    for _ in range(5):
                        co=np.polyfit(xs[inl]-1.75,res[inl],min(2,inl.sum()-1)); r=res-np.polyval(co,xs-1.75)
                        new=np.abs(r)<max(itol,3*np.median(np.abs(r[inl]))) if inl.sum()>3 else inl
                        new=np.abs(r)<itol*(1 if _>=2 else 30)
                        if new.sum()<3: break
                        inl=new
                    if inl.sum()>=minrun:
                        sgs=[fo[i][0] for i in np.where(inl)[0]]; ks=[fo[i][1] for i in np.where(inl)[0]]
                        for i in np.where(inl)[0]:
                            sg,k,o=fo[i]; hyps.append((ch,law,j,sg,k,geo)); trs.append((np.array([xs[i]]),np.array([o])))
                        acc.append((ch,law,j,int(inl.sum())))
        try: zc,res=fe.solve(hyps,trs)
        except Exception as e: print('solve failed',e); break
        msg='round %d: %d laws accepted %s; max res %.1e'%(rnd,len(acc),' '.join('%s.%s%d:%d'%a for a in acc),max(res))
        if truth is not None:
            Nn,Dd=fe.chain(zc); xg=np.linspace(1.5,2,11); Nt,Dt=truth
            e=max(np.max(np.abs(stage_F(Nn,xg)['S%d'%j][1]/stage_F(Nt,xg)['S%d'%j][1]-1)) for j in range(0,6))
            msg+='  max rel err S0-S5(N) %.1e'%e
        if verbose: print(msg,flush=True)
        if set(acc)==acc_prev: break
        acc_prev=set(acc)
    return zc

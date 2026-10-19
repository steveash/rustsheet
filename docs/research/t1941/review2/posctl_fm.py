# T-1941 review2: positive control for the per-run hidden-state fit ("FM sideband" reading) and the top-down
# integer recovery of K1's coefficients. Truth = AN2 + an integer move along the 3 most function-invisible
# directions (max |move| ~ MAXMOVE ulps, function change < 0.02 ulp). Data = the truth simulated on the real dense
# runs near 2 (P1, P1b, e10, r7, y11x, p20x). Candidate = AN2.
# Step 1: per run, coordinate ascent over 28 nudges (product and sum, N and D), objective = exact matches on the run.
# Step 2: top-down integer recovery from the fitted product nudges (phase votes), N7 first.
import numpy as np, lib2, gsimpy, sys, time
rng=np.random.default_rng(int(sys.argv[1]) if len(sys.argv)>1 else 11)
MAXMOVE=float(sys.argv[2]) if len(sys.argv)>2 else 1e5
N,D=lib2.load_cand_ND(lib2.AN2); N=np.array(N); D=np.array(D); n=7
# ---- truth along null directions
yy=np.linspace(1,2,400); Nv=np.polyval(N[::-1],yy); Dv=np.polyval(D[::-1],yy)
J=np.zeros((400,16))
for k in range(8): J[:,k]=yy**k/Nv*np.spacing(abs(N[k])); J[:,8+k]=-yy**k/Dv*np.spacing(abs(D[k]))
U,S,Vt=np.linalg.svd(J,full_matrices=False)
print("singular values (relative function change per ulp-move):",["%.1e"%v for v in S])
v=Vt[-3:]; a=rng.normal(size=3); mv=v.T@a; mv=mv/np.max(np.abs(mv))*MAXMOVE; mv=np.rint(mv)
fchg=(J@mv)/2**-52
print("truth move (ulps):",mv.astype(int)); print("max |function change| %.4f ulps"%np.max(np.abs(fchg)))
Nt=N+mv[:8]*np.spacing(abs(N)); Dt=D+mv[8:]*np.spacing(abs(D))
truth=gsimpy.Sim(Nt,Dt); cand=gsimpy.Sim(N,D)
# ---- runs
x,g,c=lib2.g_rows(); runs=[]
for nm in sorted(set(c)):
    s=np.where(c==nm)[0]; xs=np.sort(x[s])
    if len(xs)<4096 or xs[0]<1.65 or xs[-1]>=2.0: continue
    d=np.diff(xs); h=np.spacing(xs[0]); br=np.where(d!=h)[0]; segs=np.split(np.arange(len(xs)),br+1); seg=max(segs,key=len)
    if len(seg)>=8192: runs.append((nm,xs[seg[:8192]]))
print(len(runs),"runs")
# ---- step 1: per-run fit
coarse=np.arange(0,1,1/32); fine=np.arange(-3,4)/256
def fit_run(y,tgt,sweeps=4):
    pn=np.zeros(n); sn=np.zeros(n); pd=np.zeros(n); sd=np.zeros(n); pars=[pn,sn,pd,sd]
    best=cand.count(y,tgt,pn,sn,pd,sd); start=best
    order=[(w,k) for k in range(n-1,-1,-1) for w in (2,3,0,1)]   # low stages first, D then N
    for it in range(sweeps):
        improved=False
        for w,k in order:
            arr=pars[w]; v0=arr[k]; bv=v0; bc=best
            for grid in (coarse,None):
                base=bv if grid is None else 0.0; gr=(fine+base) if grid is None else grid
                for t in gr:
                    arr[k]=t%1.0; cnt=cand.count(y,tgt,pn,sn,pd,sd)
                    if cnt>bc: bc=cnt; bv=arr[k]
                arr[k]=bv
            if bc>best: best=bc; improved=True
        if not improved: break
    return start,best,pn.copy(),sn.copy(),pd.copy(),sd.copy()
def true_nudges(y,sim_t,sim_c):
    """circular mean over the run of the sub-ulp offset of the truth's product pre-value relative to the candidate's"""
    out={}
    for w,(st_,sc_) in (('N',(sim_t.trace('N',y),sim_c.trace('N',y))),('D',(sim_t.trace('D',y),sim_c.trace('D',y)))):
        cc=sim_t.N if w=='N' else sim_t.D; c0=cand.N if w=='N' else cand.D
        d=np.zeros(n)
        for k in range(n):
            if k==0: pt=cc[n]*y; pc=c0[n]*y
            else: pt=st_[:,k-1,1]*y; pc=sc_[:,k-1,1]*y
            u=np.spacing(np.abs(pc)); ph=((pt-pc)/u)%1.0
            d[k]=(np.angle(np.mean(np.exp(2j*np.pi*ph)))/(2*np.pi))%1.0
        out[w]=d
    return out
res=[]
t0=time.time()
for nm,y in runs:
    tgt=truth.G(y); st0,bst,pn,sn,pd,sd=fit_run(y,tgt); tn=true_nudges(y,truth,cand)
    res.append((nm,y[0],st0/len(y),bst/len(y),pn,sn,pd,sd,tn))
    print(f"{nm} y0={y[0]:.6f} exact {st0/len(y):.3f} -> {bst/len(y):.4f}  N prod fit  "+" ".join(f"{v:.2f}" for v in pn)
          +"  true "+" ".join(f"{v:.2f}" for v in tn['N'])+"  | D prod fit "+" ".join(f"{v:.2f}" for v in pd)+"  true "+" ".join(f"{v:.2f}" for v in tn['D'])+f"  [{time.time()-t0:.0f}s]",flush=True)
np.save(lib2.SCR+'/posctl_fm_res.npy',np.array(res,dtype=object),allow_pickle=True)
np.save(lib2.SCR+'/posctl_fm_truth.npy',np.array([mv,Nt,Dt],dtype=object),allow_pickle=True)

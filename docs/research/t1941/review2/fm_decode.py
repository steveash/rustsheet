# T-1941 review2: per-run matched-filter decoding of all stage phases (K1). For each nudge (product/sum, N/D) and
# each trial phase phi, the TEMPLATE is the output change the candidate's own chain shows when that stage's
# pre-rounding value is shifted by phi (exact simulation, so triangle/sawtooth/tie/binade cases need no law); the
# reading is the phi maximising the correlation of the template with the residual e = (Excel - candidate)/ulp.
# Unlike an exact-match count, the correlation rewards partial alignment, so it works while other stages are still
# wrong. Iterate: apply readings, recompute residual and templates. Synthetic truth as in posctl_fm.py.
import numpy as np, lib2, gsimpy, sys, time
rng=np.random.default_rng(int(sys.argv[1]) if len(sys.argv)>1 else 11)
MAXMOVE=float(sys.argv[2]) if len(sys.argv)>2 else 1e5
FIXD0=int(sys.argv[3]) if len(sys.argv)>3 else 1
RUNSEL=sys.argv[4] if len(sys.argv)>4 else 'p1'
N,D=lib2.load_cand_ND(lib2.AN2); N=np.array(N); D=np.array(D); n=7
yy=np.linspace(1,2,400); Nv=np.polyval(N[::-1],yy); Dv=np.polyval(D[::-1],yy)
J=np.zeros((400,16))
for k in range(8): J[:,k]=yy**k/Nv*np.spacing(abs(N[k])); J[:,8+k]=-yy**k/Dv*np.spacing(abs(D[k]))
cols=[c for c in range(16) if not (FIXD0 and c==8)]
U,S,Vt=np.linalg.svd(J[:,cols],full_matrices=False)
v=Vt[-3:]; a=rng.normal(size=3); mvc=v.T@a; mvc=mvc/np.max(np.abs(mvc))*MAXMOVE; mvc=np.rint(mvc)
mv=np.zeros(16); mv[cols]=mvc; fchg=(J@mv)/2**-52
print("truth move (ulps):",mv.astype(int)); print("max |function change| %.4f ulps"%np.max(np.abs(fchg)))
Nt=N+mv[:8]*np.spacing(abs(N)); Dt=D+mv[8:]*np.spacing(abs(D))
truth=gsimpy.Sim(Nt,Dt); cand=gsimpy.Sim(N,D)
x,g,c=lib2.g_rows(); runs=[]
for nm in sorted(set(c)):
    s=np.where(c==nm)[0]; xs=np.sort(x[s])
    if len(xs)<4096 or xs[0]<1.0 or xs[-1]>=2.0: continue
    d=np.diff(xs); h=np.spacing(xs[0]); br=np.where(d!=h)[0]; segs=np.split(np.arange(len(xs)),br+1); seg=max(segs,key=len)
    if len(seg)>=8192 and ((RUNSEL=='p1' and nm.startswith('kmyst-gam-p1')) or (RUNSEL=='all')): runs.append((nm,xs[seg[:8192]]))
print(len(runs),"runs")
grid=np.arange(0,1,1/64)
names=[f"{w}{t}{k+1}" for w in 'ND' for t in 'PS' for k in range(n)]
def decode(y,tgt,iters=6):
    pars=[np.zeros(n) for _ in range(4)]   # pn,sn,pd,sd
    hist=[]
    for it in range(iters):
        g0=cand.G(y,*pars); u=np.spacing(np.abs(g0)); e=(tgt-g0)/u; cnt=int((g0==tgt).sum())
        hist.append(cnt)
        order=[(w,k) for k in range(n-1,-1,-1) for w in (1,3,0,2)]   # low stages first, sums before products
        for w,k in order:
            v0=pars[w][k]; best=(-9,v0); e_now=e
            for phi in grid:
                pars[w][k]=(v0+phi)%1.0; T=(cand.G(y,*pars)-g0)/u
                if T.std()==0: sc=-1
                else: sc=np.dot(e_now-e_now.mean(),T-T.mean())/np.sqrt(np.dot(T-T.mean(),T-T.mean())*np.dot(e_now-e_now.mean(),e_now-e_now.mean()))
                if sc>best[0]: best=(sc,pars[w][k])
            pars[w][k]=best[1]
            g0=cand.G(y,*pars); e=(tgt-g0)/u
        if len(hist)>1 and hist[-1]==hist[-2]: break
    g0=cand.G(y,*pars); hist.append(int((g0==tgt).sum()))
    return pars,hist
def true_nudges(y):
    tN=truth.trace('N',y); cN=cand.trace('N',y); tD=truth.trace('D',y); cD=cand.trace('D',y); out=[]
    for (tt,cc,Ct,Cc) in ((tN,cN,Nt,N),(tD,cD,Dt,D)):
        pn=np.zeros(n); sn=np.zeros(n)
        for k in range(n):
            dS=(Ct[n]-Cc[n])*np.ones(len(y)) if k==0 else tt[:,k-1,1]-cc[:,k-1,1]
            ph=((dS*y)/np.spacing(np.abs(cc[:,k,0])))%1.0; pn[k]=(np.angle(np.mean(np.exp(2j*np.pi*ph)))/(2*np.pi))%1.0
            dP=tt[:,k,0]-cc[:,k,0]; ph=((dP+(Ct[n-1-k]-Cc[n-1-k]))/np.spacing(np.abs(cc[:,k,1])))%1.0; sn[k]=(np.angle(np.mean(np.exp(2j*np.pi*ph)))/(2*np.pi))%1.0
        out+=[pn,sn]
    return out
t0=time.time(); allres=[]
for nm,y in runs:
    tgt=truth.G(y); pars,hist=decode(y,tgt); tn=true_nudges(y)
    err=[((pars[w]-tn[w]+0.5)%1.0-0.5) for w in range(4)]
    print(f"{nm} y0={y[0]:.5f} exact {hist[0]/8192:.3f} -> {hist[-1]/8192:.3f} ({len(hist)-1} it)  |phase err| N.P "+" ".join(f"{abs(v):.2f}" for v in err[0])+"  N.S "+" ".join(f"{abs(v):.2f}" for v in err[1])+"  D.P "+" ".join(f"{abs(v):.2f}" for v in err[2])+"  D.S "+" ".join(f"{abs(v):.2f}" for v in err[3])+f"  [{time.time()-t0:.0f}s]",flush=True)
    allres.append((nm,y[0],hist,pars,tn))
np.save(lib2.SCR+f'/fm_decode_{RUNSEL}.npy',np.array([mv,Nt,Dt,allres],dtype=object),allow_pickle=True)

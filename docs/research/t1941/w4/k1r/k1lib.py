# W4 K1 per-run stage-phase reading tools (built on review2 gsim/lib2)
import sys, os, numpy as np
R2='/PATH/TO/docs/research/t1941/review2'
os.environ['SCR']='/PATH/TO/SCR'
sys.path.insert(0,R2)
import lib2, gsimpy
N0,D0=lib2.load_cand_ND(lib2.AN2); N0=np.array(N0); D0=np.array(D0); n=7
def runs(prefixes=None,minlen=4096,chunk=8192,lo=1.0,hi=2.0):
    x,g,c=lib2.g_rows(); out=[]
    m=(x>=lo)&(x<hi); x,g,c=x[m],g[m],c[m]
    for nm in sorted(set(c)):
        if prefixes and not any(nm.startswith('kmyst-gam-'+p) for p in prefixes): continue
        s=np.where(c==nm)[0]; o=np.argsort(x[s]); xs=x[s][o]; gs=g[s][o]
        d=np.diff(xs); br=np.where(d!=np.spacing(xs[:-1]))[0]
        for seg in np.split(np.arange(len(xs)),br+1):
            for a in range(0,len(seg)-minlen+1,chunk):
                ss=seg[a:a+chunk]
                if len(ss)>=minlen: out.append((nm[10:],xs[ss],gs[ss]))
    return out
WH={'N.P':0,'N.S':1,'D.P':2,'D.S':3}
def count(sim,y,t,which=None,k=None,ph=0.0):
    pars=[np.zeros(n) for _ in range(4)]
    if which is not None: pars[WH[which]][k]=ph
    return sim.count(y,t,*pars)
def scan(sim,y,t,which,k,step=1/32,refine=True):
    """best constant nudge phase (signed) and gain for stage (which, k: 0-based, 0 = top)"""
    base=count(sim,y,t); grid=np.arange(0,1,step); cs=np.array([count(sim,y,t,which,k,p) for p in grid])-base
    j=int(np.argmax(cs)); best=grid[j]; g=cs[j]
    if refine and g>0:
        fine=best+np.arange(-step,step+1e-12,step/8); cf=np.array([count(sim,y,t,which,k,p) for p in fine])-base
        jj=int(np.argmax(cf));
        top=fine[cf>=cf[jj]-max(2,0.05*cf[jj])]; best=float(np.mean(top)); g=int(cf[jj])
    return ((best+0.5)%1)-0.5, int(g), int(base)
def trace(cand_N,cand_D,which,y):
    sim=gsimpy.Sim(cand_N,cand_D); return sim.trace(which,y)

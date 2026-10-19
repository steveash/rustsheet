# T-1941 review2: (a) is the collective (d2,d3) V on the P1 runs diagnostic? Repeat the collective scan with a
# synthetic truth whose top coefficients are 1e4 ulps off along function-null directions (posctl_fm truth): if the
# per-run phases are then random, the collective profile is flat. (b) Per-run scans on Excel's P1 runs: best
# (d2,d3), gain, and the profile curvature, so the top-stage reading is tested run by run.
import numpy as np, lib2, gsimpy, sys
N,D=lib2.load_cand_ND(lib2.AN2); N=np.array(N); D=np.array(D); n=7; cand=gsimpy.Sim(N,D)
x,g,c=lib2.g_rows()
def runs_near(lo,hi,minlen=4096,prefix='kmyst-gam-p1'):
    out=[]
    for nm in sorted(set(c)):
        if not nm.startswith(prefix): continue
        s=np.where(c==nm)[0]; o=np.argsort(x[s]); xs=x[s][o]; gs=g[s][o]
        d=np.diff(xs); br=np.where(d!=np.spacing(xs[:-1]))[0]
        for seg in np.split(np.arange(len(xs)),br+1):
            if len(seg)>=minlen and lo<=xs[seg[0]]<hi: out.append((nm,xs[seg][:8192],gs[seg][:8192]))
    return out
rs=runs_near(1.70,1.97)
# (a) null-direction truth, as posctl_fm.py seed 11 with D0 fixed, MAXMOVE 2e4
rng=np.random.default_rng(11)
yy=np.linspace(1,2,400); Nv=np.polyval(N[::-1],yy); Dv=np.polyval(D[::-1],yy); J=np.zeros((400,16))
for k in range(8): J[:,k]=yy**k/Nv*np.spacing(abs(N[k])); J[:,8+k]=-yy**k/Dv*np.spacing(abs(D[k]))
cols=[cc for cc in range(16) if cc!=8]; U,S,Vt=np.linalg.svd(J[:,cols],full_matrices=False)
v=Vt[-3:]; a=rng.normal(size=3); mvc=v.T@a; mvc=np.rint(mvc/np.max(np.abs(mvc))*2e4); mv=np.zeros(16); mv[cols]=mvc
print("synthetic null truth, moves (ulps) N0..N7, D0..D7:",mv.astype(int)," max fn change %.3f ulp"%np.max(np.abs(J@mv/2**-52)))
synth=gsimpy.Sim(N+mv[:8]*np.spacing(abs(N)),D+mv[8:]*np.spacing(abs(D)))
grid=np.arange(0,1,1/16)
def scan(rs,src):
    base=0; M=np.zeros((16,16))
    for nm,y,ge in rs:
        tgt=ge if src=='excel' else synth.G(y); base+=cand.count(y,tgt)
        for i,d2 in enumerate(grid):
            for j,d3 in enumerate(grid):
                pn=np.zeros(n); pn[1]=d2; pn[2]=d3; M[i,j]+=cand.count(y,tgt,pn)
    return base,M
for src in ('synthetic','excel'):
    base,M=scan(rs,src); i0,j0=np.unravel_index(np.argmax(M),M.shape); npts=sum(len(r[1]) for r in rs)
    print(f"collective P1 [{len(rs)} runs] {src}: base {base/npts:.4f} best +{M[i0,j0]-base:.0f} at ({grid[i0]:.3f},{grid[j0]:.3f}); range {M.max()-M.min():.0f}; d2 profile "+" ".join(f"{v:+.0f}" for v in M.max(1)-base)+" | d3 "+" ".join(f"{v:+.0f}" for v in M.max(0)-base),flush=True)
print("\nper-run Excel scans (P1):")
for nm,y,ge in rs:
    base,M=scan([(nm,y,ge)],'excel'); i0,j0=np.unravel_index(np.argmax(M),M.shape)
    print(f"  {nm} y0={y[0]:.5f} exact {base/8192:.3f} best +{M[i0,j0]-base:.0f} at ({grid[i0]:.3f},{grid[j0]:.3f}) range {M.max()-M.min():.0f}  d3 profile "+" ".join(f"{v:+.0f}" for v in M.max(0)-base),flush=True)
print("\nper-run SYNTHETIC scans (P1), truth phases expected to vary run to run:")
for nm,y,ge in rs[:8]:
    base,M=scan([(nm,y,ge)],'synthetic'); i0,j0=np.unravel_index(np.argmax(M),M.shape)
    print(f"  {nm} y0={y[0]:.5f} exact {base/8192:.3f} best +{M[i0,j0]-base:.0f} at ({grid[i0]:.3f},{grid[j0]:.3f}) range {M.max()-M.min():.0f}  d3 profile "+" ".join(f"{v:+.0f}" for v in M.max(0)-base),flush=True)

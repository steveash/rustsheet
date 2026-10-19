# T-1941 review2: can the INVISIBLE top-stage product phases of K1 be read on real Excel data at the anchors, by the
# FM bump? For a group of runs sharing one anchor (the ladder rungs), scan a constant nudge d2 on N product 2 (the
# phase of S_1(a)*a) and d3 on N product 3, summing exact matches over the group; the output weight of those stages
# is 0.01-0.1 ulp, so the bump is small but the rungs share the same top-stage state. Calibration: the same scan
# with a synthetic Excel = AN2 + top-coefficient moves (N7 +3 ulps, N6 -5, N5 +2) on the same grid.
import numpy as np, lib2, gsimpy, sys
N,D=lib2.load_cand_ND(lib2.AN2); N=np.array(N); D=np.array(D); n=7; cand=gsimpy.Sim(N,D)
x,g,c=lib2.g_rows()
def runs_near(lo,hi,minlen=2048):
    out=[]
    for nm in sorted(set(c)):
        s=np.where(c==nm)[0]; o=np.argsort(x[s]); xs=x[s][o]; gs=g[s][o]
        d=np.diff(xs); br=np.where(d!=np.spacing(xs[:-1]))[0]
        for seg in np.split(np.arange(len(xs)),br+1):
            if len(seg)>=minlen and lo<=xs[seg[0]]<hi: out.append((nm,xs[seg],gs[seg]))
    return out
Nt=N.copy(); Nt[7]+=3*np.spacing(N[7]); Nt[6]-=5*np.spacing(N[6]); Nt[5]+=2*np.spacing(N[5]); synth=gsimpy.Sim(Nt,D)
grid=np.arange(0,1,1/16)
for label,lo,hi in (("anchor 1.875 (e10)",1.8749,1.8751),("anchor 2- (e3 runs ending at 2)",1.9999,2.0),("anchor 1.75 (e7)",1.7499,1.7501),("anchor 1.5 (e7)",1.4999,1.5001),("anchor 1 (e3)",1.0,1.0001),("P1 runs 1.70-1.96",1.70,1.97)):
    rs=runs_near(lo,hi); npts=sum(len(r[1]) for r in rs)
    if not rs: print(label,"no runs"); continue
    for src in ('excel','synthetic'):
        base=0; M=np.zeros((16,16))
        for nm,y,ge in rs:
            tgt=ge if src=='excel' else synth.G(y)
            base+=cand.count(y,tgt)
            for i,d2 in enumerate(grid):
                for j,d3 in enumerate(grid):
                    pn=np.zeros(n); pn[1]=d2; pn[2]=d3; M[i,j]+=cand.count(y,tgt,pn)
        i0,j0=np.unravel_index(np.argmax(M),M.shape)
        print(f"{label} [{len(rs)} runs, {npts} pts] {src}: base {base/npts:.4f}; best {M[i0,j0]/npts:.4f} (+{M[i0,j0]-base:.0f} pts) at d2={grid[i0]:.3f} d3={grid[j0]:.3f}; "
              f"bump range {(M.max()-M.min()):.0f} pts; profiles: d2 "+" ".join(f"{v:+.0f}" for v in (M.max(1)-base))+" | d3 "+" ".join(f"{v:+.0f}" for v in (M.max(0)-base)),flush=True)

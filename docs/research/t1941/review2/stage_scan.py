# T-1941 review2: per-run, per-stage phase scans on Excel's dense runs (K1, candidate AN2). For every product nudge
# (N.P1..P7, D.P1..P7) and sum nudge (N.S1..S7, D.S1..S7), a 1-D scan of the constant nudge over [0,1) in 1/16
# steps; reported: the best nudge (as a signed phase) and its gain in exact matches on the run. A gain well above
# the run's noise at a non-zero phase localises a stage whose stored value differs from Excel's by a non-integer
# number of its grid units at that run. Runs: P1/P1b (1.70-1.96), the e10 rungs at 1.875 and the e3 rungs at 2-.
import numpy as np, lib2, gsimpy, sys
N,D=lib2.load_cand_ND(lib2.AN2); N=np.array(N); D=np.array(D); n=7; cand=gsimpy.Sim(N,D)
x,g,c=lib2.g_rows()
sel=sys.argv[1] if len(sys.argv)>1 else 'p1'
def runs(minlen=4096):
    out=[]
    for nm in sorted(set(c)):
        if sel=='p1' and not nm.startswith('kmyst-gam-p1'): continue
        if sel=='anch' and not (nm.startswith('kmyst-gam-e10') or nm.startswith('kmyst-gam-e3')): continue
        s=np.where(c==nm)[0]; o=np.argsort(x[s]); xs=x[s][o]; gs=g[s][o]
        d=np.diff(xs); br=np.where(d!=np.spacing(xs[:-1]))[0]
        for seg in np.split(np.arange(len(xs)),br+1):
            if len(seg)>=minlen and (sel!='anch' or xs[seg[0]]>1.87): out.append((nm,xs[seg][:8192],gs[seg][:8192]))
    return out
grid=np.arange(0,1,1/16); names=['N.P','N.S','D.P','D.S']
print("run y0 exact | per stage: best signed phase (gain)  [stages 1..7 for N.P, N.S, D.P, D.S]")
for nm,y,ge in runs():
    base=cand.count(y,ge); out=[]
    for w in range(4):
        row=[]
        for k in range(n):
            best=(0,0.0)
            for ph in grid:
                pars=[np.zeros(n) for _ in range(4)]; pars[w][k]=ph; cnt=cand.count(y,ge,*pars)-base
                if cnt>best[0]: best=(cnt,ph)
            row.append(f"{((best[1]+0.5)%1-0.5):+.2f}({best[0]})")
        out.append(names[w]+" "+" ".join(row))
    print(f"{nm[10:]} {y[0]:.5f} {base/len(y):.3f} | "+" | ".join(out),flush=True)

# T-1941 W6: greedy joint assignment with the exact mpmath assembler (the fast linear solver stalls for far truths).
# Seed: a list of (track, hyp). Each round: for every unassigned law-like track and each of its candidates, solve
# function + ridge + (assigned + candidate) exactly; keep the candidate if every track's rms < thr; add the best
# (lowest worst-rms) candidate per round. Usage as module: grow(T, cands, seed, n, d, thr)
import numpy as np
from assemble import solve
from lawrows import rows_for, predict
from fastenum import prep
def joint_solve(T,assign,c0N,c0D,n,d,ridge=1e-2):
    rows=[]
    for ti,h in assign:
        ch,law,j,sg,kk,eo=h; x,F=prep(T[ti],12); rows+=rows_for(ch,c0N if ch=='N' else c0D,law,j,sg,kk,x,F,eoff=0)   # exponents from the (already scaled) centre chain
    N,D,_,_=solve(n,d,rows,iters=1,ridge=ridge,c0=list(c0N)+list(c0D[1:]),K=40)
    res=[]
    for ti,h in assign:
        x,F=prep(T[ti],12); res.append(np.sqrt(np.mean((predict(N,D,h[0],h[1],h[2],x)-(h[3]*F+h[4]))**2)))
    return np.array(N),np.array(D),res
def grow(T,cands,seed,c0N,c0D,n,d,thr=2e-6,rounds=10,verbose=True):
    assign=list(seed); N,D,res=joint_solve(T,assign,c0N,c0D,n,d)
    if verbose: print('seed rms',['%.1e'%r for r in res],flush=True)
    for rnd in range(rounds):
        done={ti for ti,_ in assign}; best=None
        for ti,cl in cands.items():
            if ti in done: continue
            for h in cl:
                try: N2,D2,res=joint_solve(T,assign+[(ti,h)],N,D,n,d)
                except Exception: continue
                if max(res)<thr and (best is None or max(res)<best[0]): best=(max(res),ti,h,N2,D2)
        if best is None: break
        assign.append((best[1],best[2])); N,D=best[3],best[4]
        if verbose: print('round %d: + track %d %s  worst rms %.1e  (n=%d)'%(rnd,best[1],best[2][:3],best[0],len(assign)),flush=True)
    return assign,N,D

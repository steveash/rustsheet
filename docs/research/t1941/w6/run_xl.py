# T-1941 W6: driver for the hardened pipeline on long-run data (Excel or synthetic), up to an EM chain at ~1e-7.
# Steps: per-track candidates (wide fast enumerator, exponent offsets -4..+1) -> joint skip beam -> rank the top
# hypotheses by staircase steps explained (crossings.py) -> per-peak EM -> exact track EM (mpmath assembler).
# Usage: run_xl.py PEAKS.npz TRACKS.json STAIR.txt OUTPREFIX [ntracks]
import sys, json, pickle, numpy as np
sys.path.insert(0,sys.path[0])
from fastenum import FastEnum, prep, joint_beam_skip
from crossings import chain_crossings, stair_steps
from assemble import solve
from lawrows import rows_for
import peakem, trackem
pk,trf,stf,out=sys.argv[1:5]; ntr=int(sys.argv[5]) if len(sys.argv)>5 else 16
z=np.load(pk); T=json.load(open(trf)); T=sorted(T,key=lambda t:-len(t['F']))[:ntr]
fe=FastEnum(9,thr=1e-8,ridge=1e-10,ftol=3e-16); tr=[prep(t,12) for t in T]
C=[]
for i,(x,F) in enumerate(tr):
    top=fe.single(x,F,eoffs=(-4,-3,-2,-1,0,1),top=10**6); fl=top[0][0]; C.append((fl,[h for r,h in top if r<3*fl+1e-7]))
    print('track %2d len %2d floor %.1e ncand %d'%(i,len(T[i]['F']),fl,len(C[-1][1])),flush=True)
use=[i for i in range(len(tr)) if C[i][0]<1e-5]
B=joint_beam_skip(fe,[tr[i] for i in use],[C[i][1] for i in use],beam=150,thr=2e-6)
st=stair_steps(stf); print('staircase steps:',st)
res=[]
for a,s,ns in B[:12]:
    act=[(use[i],h) for i,h in enumerate(a) if h is not None]
    if not act: continue
    zc,_=fe.solve([h for _,h in act],[tr[i] for i,_ in act]); N,D=fe.chain(zc); cr=chain_crossings(N,D)
    hits=sum(1 for xs,_ in st if any(abs(xs-xc)<0.003 for xc,_ in cr)); res.append((hits,-s,act))
    print('hyp %s rms %.1e skips %d: steps explained %d/%d'%([h[:3] for _,h in act],s,ns,hits,len(st)),flush=True)
res.sort(key=lambda t:(-t[0],-t[1]))
best=res[0][2]; geo=best[0][1][5]; print('chosen',best)
seed=[h for _,h in best]; seedt=[tr[i] for i,_ in best]
zc=peakem.run(fe,z,seed,seedt,rounds=2,tol0=1e-3)
N,D=fe.chain(zc)
for rnd in range(3):
    rows=[];acc=[]
    for ti,(x,F) in enumerate([prep(t,12) for t in json.load(open(trf))]):
        bl=trackem.best_law(N,D,x,F,9)
        if bl[0][0]<2e-3/(3**rnd) and bl[1][0]>3*bl[0][0]:
            ch,law,j,sg,kk=bl[0][1]; rows+=rows_for(ch,N if ch=='N' else D,law,j,sg,kk,x,F); acc.append((ti,'%s.%s%d'%(ch,law,j),bl[0][0]))
    N1,D1,_,_=solve(9,9,rows,iters=1,ridge=1e-2,c0=list(N)+list(D[1:]),K=40); N,D=np.array(N1),np.array(D1)
    print('track-EM round %d: %d tracks %s'%(rnd,len(acc),' '.join('%d:%s(%.0e)'%a for a in acc)),flush=True)
open(out+'_EM.txt','w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')
print('wrote',out+'_EM.txt')

# T-1941 W6: exact track-EM on Excel from a joint chain, then the staircase/crossing check.
# Usage: xl_em.py CHAIN TRACKS STAIR OUT n d
import sys,json; sys.path.insert(0,sys.path[0])
import numpy as np
from assemble import solve
from lawrows import rows_for
from fastenum import prep
from synth import read_chain
from crossings import chain_crossings, stair_steps
import trackem
N,D=read_chain(sys.argv[1]); N=np.array(N)/D[0]; D=np.array(D)/D[0]; n,d=int(sys.argv[5]),int(sys.argv[6])
T=json.load(open(sys.argv[2])); tr=[prep(t,12) for t in T]
for rnd,tol in enumerate((2e-3,6e-4,2e-4,6e-5,2e-5)):
    rows=[];acc=[]
    for ti,(x,F) in enumerate(tr):
        bl=trackem.best_law(N,D,x,F,max(n,d))
        if bl[0][0]<tol and bl[1][0]>3*bl[0][0]:
            ch,law,j,sg,kk=bl[0][1]; rows+=rows_for(ch,N if ch=='N' else D,law,j,sg,kk,x,F); acc.append((ti,'%s.%s%d'%(ch,law,j),bl[0][0]))
    N1,D1,_,_=solve(n,d,rows,iters=1,ridge=1e-2,c0=list(N)+list(D[1:]),K=40); N,D=np.array(N1),np.array(D1)
    print('EM round %d tol %.0e: %d tracks: %s'%(rnd,tol,len(acc),' '.join('%d:%s(%.0e)'%a for a in acc)),flush=True)
st=stair_steps(sys.argv[3]); cr=chain_crossings(N,D)
print('staircase steps:',' '.join('%.4f'%s for s,_ in st))
for xs,dv in st:
    near=[(round(xc,4),nm) for xc,nm in cr if abs(xs-xc)<0.004]
    print('  step %.4f (%+.2f): crossings %s'%(xs,dv,near))
open(sys.argv[4],'w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n'); print('wrote',sys.argv[4])

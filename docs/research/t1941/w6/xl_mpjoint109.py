# T-1941 W6: exact-assembler greedy joint assignment on Excel's long-run tracks at degree (10,9), seeded with the
# fast scan's best joint triple (D.T5, N.T4, N.T5).
import sys,json; sys.path.insert(0,'/home/user/rustsheet/docs/research/t1941/w6')
import numpy as np
from fastenum import FastEnum, prep
import mpjoint
S='/tmp/claude-0/-home-user-rustsheet/4ac34ca4-6a1e-5914-b8bd-912826bc8336/scratchpad/'
T=json.load(open(S+'Lxl_tracks.json')); T=sorted(T,key=lambda t:-len(t['F']))[:18]
fe=FastEnum(10,thr=1e-8,ridge=1e-10,ftol=3e-16,d=9)
law=[0,2,3,4,8,9,10,13,14,16,17]; cands={}
for i in law:
    x,F=prep(T[i],12); top=fe.single(x,F,eoffs=(-4,-3,-2,-1,0,1),top=10**6); fl=top[0][0]; cands[i]=[h for r,h in top if r<3*fl+1e-7][:25]
want={0:('D','T',5),13:('N','T',4),17:('N','T',5)}
import itertools, trackem
best=None
for combo in itertools.product(*[[c for c in cands[i] if c[:3]==key] for i,key in want.items()]):
    try: z,res=fe.solve(list(combo),[prep(T[i],12) for i in want])
    except Exception: continue
    if best is None or max(res)<best[0]: best=(max(res),z)
print('fast seed rms %.1e'%best[0],flush=True)
N0,D0=fe.chain(best[1]); seed=[]
for i,key in want.items():
    x,F=prep(T[i],12); bl=[b for b in trackem.best_law(N0,D0,x,F,10) if b[1][:3]==key]; seed.append((i,bl[0][1]+(0,)))
a,N,D=mpjoint.grow(T,cands,seed,N0,D0,10,9,thr=2e-6,rounds=10)
print('final',[(i,h[:3]) for i,h in a],flush=True)
open(S+'xl_mpjoint_109.txt','w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')

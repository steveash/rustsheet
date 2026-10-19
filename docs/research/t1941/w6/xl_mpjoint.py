import sys,json; sys.path.insert(0,'/home/user/rustsheet/docs/research/t1941/w6')
import numpy as np
from fastenum import FastEnum, prep
import mpjoint
S='/tmp/claude-0/-home-user-rustsheet/4ac34ca4-6a1e-5914-b8bd-912826bc8336/scratchpad/'
T=json.load(open(S+'Lxl_tracks.json')); T=sorted(T,key=lambda t:-len(t['F']))[:18]
fe=FastEnum(9,thr=1e-8,ridge=1e-10,ftol=3e-16)
law=[0,2,4,8,9,13,14,16,17]; cands={}
for i in law:
    x,F=prep(T[i],12); top=fe.single(x,F,eoffs=(-4,-3,-2,-1,0,1),top=10**6); fl=top[0][0]; cands[i]=[h for r,h in top if r<3*fl+1e-7][:25]
import itertools, trackem
for seedname,want in (('D',{0:('D','T',5),4:('D','Pn1',5)}),('N',{13:('N','T',4),17:('N','T',5)})):
    best=None
    for combo in itertools.product(*[[c for c in cands[i] if c[:3]==key] for i,key in want.items()]):
        try: z,res=fe.solve(list(combo),[prep(T[i],12) for i in want])
        except Exception: continue
        if best is None or max(res)<best[0]: best=(max(res),z)
    print('== seed',seedname,'fast rms %.1e'%best[0],flush=True)
    N0,D0=fe.chain(best[1]); seed2=[]
    for i,key in want.items():   # sign/int from the fast joint chain's own prediction of this law
        x,F=prep(T[i],12); bl=[b for b in trackem.best_law(N0,D0,x,F,9) if b[1][:3]==key]; seed2.append((i,bl[0][1]+(0,)))
    a,N,D=mpjoint.grow(T,cands,seed2,N0,D0,9,9,thr=2e-6,rounds=8)
    print('final',[(i,h[:3]) for i,h in a],flush=True)
    open(S+'xl_mpjoint_%s.txt'%seedname,'w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')

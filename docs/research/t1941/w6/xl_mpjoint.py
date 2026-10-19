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
for seedname,seed in (('D',[(0,('D','T',5,1,0,-2)),(4,('D','Pn1',5,-1,1,-2))]),('N',[(13,('N','T',4,1,0,-2)),(17,('N','T',5,1,0,-2))])):
    # seed centre from the fast solver
    hs=[h for _,h in seed]; z,_=fe.solve(hs,[prep(T[i],12) for i,_ in seed]); N0,D0=fe.chain(z)
    print('== seed',seedname,flush=True)
    # fix the seed's own sign/int using the fast candidates (take the matching law entry from cands)
    seed2=[]
    for i,h in seed:
        m=[c for c in cands[i] if c[:3]==h[:3]]; seed2.append((i,m[0] if m else h))
    a,N,D=mpjoint.grow(T,cands,seed2,N0,D0,9,9,thr=2e-6,rounds=8)
    print('final',[(i,h[:3]) for i,h in a],flush=True)
    open(S+'xl_mpjoint_%s.txt'%seedname,'w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')

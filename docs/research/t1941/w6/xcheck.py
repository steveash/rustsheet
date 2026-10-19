# T-1941 W6: for a joint assignment of Excel tracks, solve the chain and count how many of W4's unassigned Excel
# peaks (4-decimal) and the D track points the chain's full law set explains, vs a shuffled-x null.
import sys, json, numpy as np
sys.path.insert(0,sys.path[0]); from assemble import solve; from lawrows import rows_for; from stagelaws import stage_F, quotient_F
T=json.load(open(sys.argv[1])); U=[l.split() for l in open(sys.argv[2])]
def alias(F): F=np.mod(F,1.0); return np.minimum(F,1-F)
def laws(N,D,x):
    fam={}
    for ch,c in (('N',N),('D',D)):
        for nm,(F,v,vd) in stage_F(np.array(c),np.array([x])).items(): fam[ch+'.'+nm]=alias(F)[0]
    for nm,(F,v,vd) in quotient_F(np.array(N),np.array(D),np.array([x])).items(): fam['R.'+nm]=alias(F)[0]
    return fam
N0,D0,_,_=solve(9,9,[],iters=1,K=40); c0=N0+D0[1:]
for spec in sys.argv[3:]:
    rows=[]
    for part in spec.split(';'):
        ti,ch,law,j,sg,kk,eo=part.split(','); ti=int(ti); t=T[ti]; x=np.array(t['x'])+32768*2.0**-52
        rows+=rows_for(ch,N0 if ch=='N' else D0,law,int(j),int(sg),int(kk),x,np.array(t['F']),eoff=int(eo))
    N,D,_,_=solve(9,9,rows,iters=1,ridge=100,c0=c0,K=40)
    hit=0; tot=0; nul=0; names={}
    rng=np.random.default_rng(0); xs=[float(u[0]) for u in U]
    for u in U:
        x=float(u[0]); fam=laws(N,D,x); fv=np.array(list(fam.values())); nm=list(fam.keys())
        famn=laws(N,D,xs[rng.integers(len(xs))]+0.0); fn=np.array(list(famn.values()))
        for f in map(float,u[1:]):
            d=np.abs(fv-f); tot+=1
            if d.min()<1.2e-4: hit+=1; names[nm[d.argmin()]]=names.get(nm[d.argmin()],0)+1
            nul+= np.abs(fn-f).min()<1.2e-4
    tD=[t for t in T if t['name']=='D'][0]; dh=0
    for x,f in zip(tD['x'],tD['F']):
        fam=laws(N,D,x+32768*2.0**-52); dh+= min(abs(v-f) for v in fam.values())<2e-5
    print('%s: unassigned explained %d/%d (null %d); D points %d/%d; %s'%(spec,hit,tot,nul,dh,len(tD['x']),sorted(names.items(),key=lambda kv:-kv[1])[:8]))

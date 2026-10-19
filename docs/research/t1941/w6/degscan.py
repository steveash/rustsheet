# T-1941 W6: degree scan on long-run tracks: for each (n,n), per-track candidates (wide fast enumerator, eoffs -4..1)
# and the joint skip beam; report how many tracks fit jointly (fewest skips) and the best rms.
import sys, json, numpy as np
sys.path.insert(0,sys.path[0]); from fastenum import FastEnum, prep, joint_beam_skip
T=json.load(open(sys.argv[1])); T=sorted(T,key=lambda t:-len(t['F']))[:int(sys.argv[3])]; tr=[prep(t,12) for t in T]
for n in [int(v) for v in sys.argv[2].split(',')]:
    fe=FastEnum(n,thr=1e-8,ridge=1e-10,ftol=3e-16); C=[]
    for x,F in tr:
        top=fe.single(x,F,eoffs=(-4,-3,-2,-1,0,1),top=10**6); fl=top[0][0]; C.append((fl,[h for r,h in top if r<3*fl+1e-7]))
    use=[i for i in range(len(tr)) if C[i][0]<1e-5]
    B=joint_beam_skip(fe,[tr[i] for i in use],[C[i][1] for i in use],beam=150,thr=2e-6)
    a,s,ns=B[0]
    print('DEGREE (%d,%d): law-like tracks %d (%s); joint best: %d fitted, rms %.1e: %s'%(n,n,len(use),use,len(use)-ns,s,[(use[i],h[:3]) for i,h in enumerate(a) if h is not None]),flush=True)

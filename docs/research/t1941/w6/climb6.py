# T-1941 W6: exact-match coordinate climb for the (1.5,2] chain. Score = fraction of points whose simulated R lies in
# Excel's (or synthetic) R candidate interval [Rlo, Rhi]. Moves: each coefficient +-2^k ulps, k = 0..kmax, greedy,
# repeated until no gain. Usage: climb6.py RFILE CHAIN OUT npts kmax rounds [prefix]
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner; from synth import read_chain, simR
rows=[l.split() for l in open(sys.argv[1]) if len(sys.argv)<=7 or l.split()[5].startswith(sys.argv[7])]
rng=np.random.default_rng(0); idx=rng.choice(len(rows),min(int(sys.argv[4]),len(rows)),replace=False)
x=np.array([float.fromhex(rows[i][0]) for i in idx]); lo=np.array([float.fromhex(rows[i][2]) for i in idx]); hi=np.array([float.fromhex(rows[i][3]) for i in idx])
N,D=read_chain(sys.argv[2]); N=[v/D[0] for v in N]; D=[v/D[0] for v in D]
kmax=int(sys.argv[5])
def score(N,D):
    R=simR(N,D,x); return np.mean((R>=lo)&(R<=hi))
def nb(v,k):
    if k==0: return v
    i=np.frombuffer(np.float64(v).tobytes(),np.int64)[0]+(k if v>=0 else -k); return float(np.frombuffer(np.int64(i).tobytes(),np.float64)[0])
cur=score(N,D); print('start %.4f'%cur,flush=True)
for rnd in range(int(sys.argv[6])):
    imp=False
    for ch in ('N','D'):
        c=N if ch=='N' else D
        for j in range(0 if ch=='N' else 1,len(c)):
            best=(cur,0)
            for k in range(kmax+1):
                for s in (1,-1):
                    cc=list(c); cc[j]=nb(c[j],s*2**k)
                    v=score(cc,D) if ch=='N' else score(N,cc)
                    if v>best[0]: best=(v,s*2**k)
            if best[1]:
                c[j]=nb(c[j],best[1]); cur=best[0]; imp=True
                print('  %s%d %+d -> %.4f'%(ch,j,best[1],cur),flush=True)
    print('round %d: %.4f'%(rnd,cur),flush=True)
    if not imp: break
open(sys.argv[3],'w').write('N '+' '.join(float(v).hex() for v in N)+'\nD '+' '.join(float(v).hex() for v in D)+'\n')

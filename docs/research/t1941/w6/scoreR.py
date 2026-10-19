# T-1941 W6: exact-R score of chains: fraction of sampled points whose simulated R lies in the data's [Rlo, Rhi], and
# the median relative R error. Usage: scoreR.py RFILE CHAIN... [--n 20000] [--prefix kmyst-gam-L1]
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from lib6 import st, L, horner; from synth import read_chain
a=sys.argv[1:]; n=20000; pref=''
if '--n' in a: i=a.index('--n'); n=int(a[i+1]); del a[i:i+2]
if '--prefix' in a: i=a.index('--prefix'); pref=a[i+1]; del a[i:i+2]
rows=[l.split() for l in open(a[0]) if l.split()[5].startswith(pref)]
idx=np.random.default_rng(0).choice(len(rows),min(n,len(rows)),replace=False)
x=np.array([float.fromhex(rows[i][0]) for i in idx]); lo=np.array([float.fromhex(rows[i][2]) for i in idx]); hi=np.array([float.fromhex(rows[i][3]) for i in idx])
for f in a[1:]:
    N,D=read_chain(f); N=[v/D[0] for v in N]; D=[v/D[0] for v in D]
    R=st(L(horner(N[::-1],x))/L(horner(D[::-1],x)))
    print('%-40s exact %.4f  median rel err %.2e'%(f.split('/')[-1],np.mean((R>=lo)&(R<=hi)),np.median(np.abs(R-(lo+hi)/2)/np.abs(lo))))

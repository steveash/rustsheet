# T-1941 W6: apply the lattice's recovered integer offsets ("rec  m:" line of latsolve output) to the candidate chain.
# Usage: applym.py CAND LATOUT OUT
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from synth import read_chain
N,D=read_chain(sys.argv[1]); N=[v/D[0] for v in N]; D=[v/D[0] for v in D]
m=eval([l for l in open(sys.argv[2]) if l.startswith('rec  m:')][0].split(':',1)[1])
def nb(v,k):
    i=np.frombuffer(np.float64(v).tobytes(),np.int64)[0]+(int(k) if v>=0 else -int(k)); return float(np.frombuffer(np.int64(i).tobytes(),np.float64)[0])
N2=[nb(c,k) for c,k in zip(N,m[:len(N)])]; D2=[1.0]+[nb(c,k) for c,k in zip(D[1:],m[len(N):])]
open(sys.argv[3],'w').write('N '+' '.join(v.hex() for v in N2)+'\nD '+' '.join(v.hex() for v in D2)+'\n')

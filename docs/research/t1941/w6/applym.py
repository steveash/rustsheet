# T-1941 W6: apply the lattice's recovered integer offsets ("rec  m:" line of latsolve output) to the candidate chain.
# Usage: applym.py CAND LATOUT OUT
import sys, numpy as np
sys.path.insert(0,sys.path[0]); from synth import read_chain, nrm
N,D=read_chain(sys.argv[1]); N=[v/nrm(D) for v in N]; D=[v/nrm(D) for v in D]
m=eval([l for l in open(sys.argv[2]) if l.startswith('rec  m:')][0].split(':',1)[1])
def nb(v,k):
    # m is in ulps of the candidate value (latsolve's linearisation), so add the value k*ulp(v); stepping the bit
    # pattern instead is wrong once v crosses a binade (each ulp past the crossing is 2x or 0.5x as large).
    return float(v+int(k)*np.spacing(abs(v)))
N2=[nb(c,k) for c,k in zip(N,m[:len(N)])]; D2=[1.0]+[nb(c,k) for c,k in zip(D[1:],m[len(N):])]
open(sys.argv[3],'w').write('N '+' '.join(v.hex() for v in N2)+'\nD '+' '.join(v.hex() for v in D2)+'\n')

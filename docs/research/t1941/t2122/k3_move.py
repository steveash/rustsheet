"""Apply integer-ulp moves to N7, N8, N9 of a K3 set (T-2122 run 3). usage: k3_move.py W5DIR IN OUT dN7 dN8 dN9"""
import sys, numpy as np
sys.path.insert(0, sys.argv[1])
def ld(f):
    l = open(f).read().split('\n'); return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]
N, D = ld(sys.argv[2])
for k, u in zip((7, 8, 9), map(float, sys.argv[4:7])):
    N[k] += round(u)*np.spacing(abs(N[k]))
open(sys.argv[3], 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')

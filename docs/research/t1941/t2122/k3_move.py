"""Apply integer-ulp moves to a K3 set (T-2122 runs 3-4). usage: k3_move.py W5DIR IN OUT dN7 dN8 dN9 (run 3), or
k3_move.py W5DIR IN OUT N6=ulps D5=ulps ... (any coefficient, run 4)"""
import sys, numpy as np
sys.path.insert(0, sys.argv[1])
def ld(f):
    l = open(f).read().split('\n'); return [np.array([float.fromhex(v) for v in l[i].split()]) for i in (0, 1)]
N, D = ld(sys.argv[2])
args = sys.argv[4:]
moves = [a.split('=') for a in args] if all('=' in a for a in args) else [('N%d' % k, u) for k, u in zip((7, 8, 9), args)]
for key, u in moves:
    C = N if key[0] == 'N' else D; k = int(key[1:])
    C[k] += round(float(u))*np.spacing(abs(C[k]))
open(sys.argv[3], 'w').write(' '.join(v.hex() for v in N)+'\n'+' '.join(v.hex() for v in D)+'\n')

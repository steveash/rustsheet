# T-1941 W6: add whole ulps to named coefficients of a chain. Usage: bump.py IN OUT N6=+1534 [D7=-3 ...]
import sys, numpy as np
sys.path.insert(0, sys.path[0])
from synth import read_chain, nrm
N, D = read_chain(sys.argv[1])
N = list(np.array(N) /nrm(D))
D = list(np.array(D) /nrm(D))
for a in sys.argv[3:]:
    k, v = a.split("=")
    arr = N if k[0] == "N" else D
    j = int(k[1:])
    arr[j] = arr[j] + int(v) * np.spacing(abs(arr[j]))
open(sys.argv[2], "w").write("N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n")

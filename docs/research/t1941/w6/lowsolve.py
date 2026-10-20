# T-1941 W6: re-solve chosen coefficients (e.g. the low ones, N0..N4, D1, D2) by least squares on the function rows
# with all others held, iterating to convergence (the rows are linear in the offsets around the current chain).
# Usage: lowsolve.py CHAIN OUT "N0,N1,N2,N3,N4,D1,D2" [iters]   env DATAF/DATAN (Excel-R-anchored rows instead of lgamma)
import sys, os, numpy as np
sys.path.insert(0, sys.path[0])
from synth import read_chain, nrm
from latsolve import func_rows, data_rows

chain, out, free = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
it = int(sys.argv[4]) if len(sys.argv) > 4 else 4
N, D = read_chain(chain)
N = list(np.array(N) /nrm(D))
D = list(np.array(D) /nrm(D))
nN = len(N)
cols = [(int(f[1:]) if f[0] == "N" else nN - 1 + int(f[1:])) for f in free]
for i in range(it):
    fr = data_rows(N, D, os.environ["DATAF"], int(os.environ.get("DATAN", "400"))) if os.environ.get("DATAF") else func_rows(N, D)
    A = np.array([[float(g[c]) for c in cols] for g, _ in fr])
    b = -np.array([float(v) for _, v in fr])
    sc = np.max(np.abs(A), axis=0)
    z, *_ = np.linalg.lstsq(A / sc, b, rcond=None)
    z = np.round(z / sc)
    for f, k in zip(free, z):
        arr = N if f[0] == "N" else D
        j = int(f[1:])
        arr[j] = arr[j] + k * np.spacing(abs(arr[j]))
    print("iter %d: residual before max %.1e; moves %s" % (i, np.max(np.abs(b)), " ".join("%s %+.3g" % (f, k) for f, k in zip(free, z))), flush=True)
fr = func_rows(N, D)
print("final function residual max %.1e" % max(abs(float(v)) for _, v in fr))
open(out, "w").write("N " + " ".join(float(v).hex() for v in N) + "\nD " + " ".join(float(v).hex() for v in D) + "\n")

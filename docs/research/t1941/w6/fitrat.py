# T-1941 W6: linearised least-squares rational N/D (raw x, D0 = 1) to the kernel's exact R (kern.Rf) at Chebyshev
# nodes, rounded to doubles: a synthetic truth for controls on any kernel. Usage: KERNEL=k2 fitrat.py n d OUT
import sys, mpmath as mp
sys.path.insert(0, sys.path[0])
import kern
mp.mp.dps = 60
n, d = int(sys.argv[1]), int(sys.argv[2])
K = 4 * (n + d + 1)
c, h = (mp.mpf(kern.LO) + mp.mpf(kern.HI)) / 2, (mp.mpf(kern.HI) - mp.mpf(kern.LO)) / 2
xs = [c + h * mp.cos(mp.pi * (k + mp.mpf(1) / 2) / K) for k in range(K)]
A = mp.matrix(K, n + d + 1)
b = mp.matrix(K, 1)
for i, x in enumerate(xs):
    r = kern.Rf(x)
    for k in range(n + 1):
        A[i, k] = x**k / r
    for k in range(1, d + 1):
        A[i, n + k] = -(x**k)
    b[i] = 1
sol = mp.lu_solve(A.T * A, A.T * b)
N = [float(sol[k]) for k in range(n + 1)]
D = [1.0] + [float(sol[n + k]) for k in range(1, d + 1)]
err = max(abs(mp.polyval([mp.mpf(v) for v in N[::-1]], x) / mp.polyval([mp.mpf(v) for v in D[::-1]], x) / kern.Rf(x) - 1) for x in xs)
print("max relative error of the rounded rational: %.1e" % err)
open(sys.argv[3], "w").write("N " + " ".join(v.hex() for v in N) + "\nD " + " ".join(v.hex() for v in D) + "\n")

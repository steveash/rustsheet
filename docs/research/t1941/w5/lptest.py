# LP: min max|delta_i| (coefficient roundings, in ulps) s.t. sum delta_i*sens_i(x) = Delta_E(x) +- tol.
# If the optimum exceeds 0.5 ulp, no parent with negligible approximation error can explain Excel's function.
import numpy as np, mpmath as mp, sys
from scipy.optimize import linprog
from sc import candidate

mp.mp.dps = 40
N = candidate("N3")
D = candidate("D3")
dE = np.load(sys.argv[1] if len(sys.argv) > 1 else "deltaE.npy")
xs = dE[:, 0]
target = dE[:, 3]


def F(n, d, x):
    return mp.polyval(n[::-1], x) / mp.polyval(d[::-1], x)


nm = [mp.mpf(v) for v in N]
dm = [mp.mpf(v) for v in D]
base = [F(nm, dm, mp.mpf(x)) for x in xs]
S = []
lab = []
for ch in "ND":
    for i in range(10):
        arr = list(nm) if ch == "N" else list(dm)
        if arr[i] == 0:
            continue
        u = mp.mpf(np.spacing(abs(float(arr[i]))))
        arr[i] += u
        n2, d2 = (arr, dm) if ch == "N" else (nm, arr)
        S.append([float(F(n2, d2, mp.mpf(x)) / bb - 1) for x, bb in zip(xs, base)])
        lab.append(ch + str(i))
S = np.array(S).T * 1e17  # bins x coefs, units of 1e-17
target = target * 1e17


def lp(target, w):
    k = S.shape[1]
    c = np.r_[np.zeros(k), 1]
    A = []
    bvec = []
    for i in range(k):
        e = np.zeros(k + 1)
        e[i] = 1
        e[-1] = -1
        A.append(e)
        bvec.append(0)
        e = np.zeros(k + 1)
        e[i] = -1
        e[-1] = -1
        A.append(e)
        bvec.append(0)
    for j in range(len(xs)):
        A.append(np.r_[S[j], 0])
        bvec.append(target[j] + w[j])
        A.append(np.r_[-S[j], 0])
        bvec.append(-target[j] + w[j])
    r = linprog(
        c, A_ub=np.array(A), b_ub=np.array(bvec), bounds=[(None, None)] * k + [(0, None)], method="highs"
    )
    if r.status != 0: print("LP status", r.status, r.message); return np.full(S.shape[1]+1, np.nan)
    return r.x


if __name__ == "__main__":
    for tol, skipbumps in [(3e-17, True), (5e-17, True), (3e-17, False), (8e-17, False)]:
        w = np.full(len(xs), tol*1e17)
        if skipbumps:
            w[((xs > 0.82) & (xs < 0.86)) | ((xs > 1.12) & (xs < 1.16))] = 20.0
        x = lp(target, w)
        print("tol %.0e bumps-skipped %s: min max|delta| = %.3f ulp" % (tol, skipbumps, x[-1]))
        if tol == 3e-17 and skipbumps:
            print("   delta:", " ".join("%s:%+.2f" % (l, v) for l, v in zip(lab, x[:-1])))

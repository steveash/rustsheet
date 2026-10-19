"""Function-preserving synthetic truth: base + random combination of the softest SVD directions (T-1941 W5 part 5)."""
import numpy as np
def make_truth(N0, D0, seed=1, tsize=1e5, kmin=15):
    xs = np.linspace(0.7, 1.5, 200).astype(np.longdouble)
    Nv = np.polyval(N0[::-1].astype(np.longdouble), xs); Dv = np.polyval(D0[::-1].astype(np.longdouble), xs)
    cols = [np.spacing(abs(N0[i]))*xs**i/Nv for i in range(len(N0))] + [-np.spacing(abs(D0[i]))*xs**i/Dv for i in range(1, len(D0))]
    M = np.array(cols, dtype=float).T; U, s, Vt = np.linalg.svd(M, full_matrices=False)
    rng = np.random.default_rng(seed)
    mv = sum(rng.normal()*Vt[k] for k in range(kmin, len(s)))
    mv = np.round(mv/np.abs(mv).max()*tsize)
    Nt = N0.copy(); Dt = D0.copy()
    for i in range(len(N0)): Nt[i] += mv[i]*np.spacing(abs(N0[i]))
    for k, i in enumerate(range(1, len(D0))): Dt[i] += mv[len(N0)+k]*np.spacing(abs(D0[i]))
    return Nt, Dt, mv, float(np.abs(M@mv).max())

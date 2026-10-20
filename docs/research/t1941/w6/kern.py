# T-1941 W6: the kernel-specific pieces of the W6 tools, selected by env KERNEL (default k6 = GAMMALN.PRECISE on
# (1.5,2]: B = fl((x-1)*fl((x-2)*R)); k2 = (2,3]: B = fl((x-2)*R), W2's structure; k1 = W4's GAMMA G = fl(N/D)). Everything else (stage laws,
# ladder reader, lattice) is kernel-independent and takes the chain degrees from the chain file.
#   Rf(z)        exact R (mpmath): what the residuals and function rows are measured against
#   outer(x, R)  the stored outer ops applied to the stored R (x87 double rounding, like the chain)
#   extract(x, B, span)  all doubles R within +-span of the naive inverse with outer(x, R) == B -> (lo, hi, n)
#   LO, HI       the x range (function-row nodes, rext row selection)
import os, numpy as np, mpmath as mp
from lib6 import st, L

KERNEL = os.environ.get("KERNEL", "k6")
if KERNEL == "k6":
    LO, HI = 1.5, 2.0

    def Rf(z):
        return mp.loggamma(z) / ((z - 1) * (z - 2))

    def outer(x, R):
        return st(L(x - 1.0) * L(st(L(x - 2.0) * L(R))))

    def naive(x, B):
        return B / ((x - 1.0) * (x - 2.0))

elif KERNEL == "k2":
    LO, HI = 2.0, 3.0

    def Rf(z):
        return mp.loggamma(z) / (z - 2)

    def outer(x, R):
        return st(L(x - 2.0) * L(R))

    def naive(x, B):
        return B / (x - 2.0)

elif KERNEL == "k1":
    # W4's K1 (GAMMA): G = fl(N/D) directly on y in [1,2), no outer op; Rf assumed Gamma(y) (W4 to confirm). Chains with
    # D0 = -1/16 are fine: the tools normalise by D0, an exact power-of-two scaling (every rounding is unchanged).
    LO, HI = 1.0, 2.0

    def Rf(z):
        return mp.gamma(z)

    def outer(x, R):
        return R

    def naive(x, B):
        return B

else:
    raise SystemExit("unknown KERNEL %r (k6, k2, k1)" % KERNEL)


def extract(x, B, span=5):
    R0 = naive(x, B)
    H = []
    for k in range(-span, span + 1):
        Rk = R0.copy()
        for _ in range(abs(k)):
            Rk = np.nextafter(Rk, np.inf if k > 0 else -np.inf)
        H.append(np.where(outer(x, Rk) == B, Rk, np.nan))
    H = np.array(H)
    n = np.sum(~np.isnan(H), 0)
    with np.errstate(all="ignore"):
        return np.nanmin(H, 0), np.nanmax(H, 0), n

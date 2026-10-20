# K2 (GAMMALN on (2,3]) reference model "3p/2p" (W2 + reviewer 3, 2026-10-20 06:20 UTC). Exact-match vs Excel:
# 14 long runs 0.9089, 1,024 dense clusters 0.9111 (grid_a stored Horner: 0.739 / 0.757). Not bit-exact; see
# HANDOFF 12.17-12.18. numpy longdouble = x87 80-bit; sd = store to double.
import sys
import numpy as np
sys.path.insert(0, 'docs/research/t1941/review3')
from spec import p, q, load  # grid_a coefficients (high-order first)

LD = np.longdouble
U = 2.0 ** -54
C_N, C_D = -3128 * U, 552 * U          # hand-off c/x terms (N at stage 4, D at stage 3)

def sd(v):
    return np.asarray(v, dtype=LD).astype(np.float64).astype(LD)

def chain(c, x, K, cc):
    """Stored x87 Horner; at stage K the c/x term enters the product before its rounding and is removed
    exactly at the next constant (c[K+1] - cc), so the polynomial is algebraically unchanged."""
    s = LD(c[0]) + 0 * x
    for i in range(1, len(c)):
        ci = LD(c[i]) - (LD(cc) if i == K + 1 else 0)
        s = sd(sd(s * x + LD(cc) / x) + ci) if i == K else sd(sd(s * x) + ci)
    return s

def k2(x):
    x = np.asarray(x).astype(LD)
    return sd(sd(x - 2) * sd(chain(p, x, 4, C_N) / chain(q, x, 3, C_D))).astype(np.float64)

if __name__ == '__main__':
    for case in sys.argv[1:]:
        X, Y = load(case)
        print(case, len(X), round(float(np.mean(k2(X) == Y)), 4))

# plateau-reader positive control for K1 (run from review/, uses k1lib): step heights at AN2 sum crossings vs low-bit perturbation of the stage constant
import sys

sys.path.insert(0, ".")
from k1lib import *

N, D = load_cand(sys.argv[1])
specs = [
    ("N", 1, 1.0519),
    ("N", 3, 1.7029),
    ("N", 4, 1.3403),
    ("N", 6, 1.5445),
    ("D", 2, 1.2944),
    ("D", 4, 1.1799),
    ("D", 6, 1.3694),
]
n = 16384


def side(c, sgn):
    y0 = c + 4e-4 if sgn > 0 else np.nextafter(c - 4e-4, 0)
    out = np.empty(n)
    v = y0
    for i in range(n):
        out[i] = v
        v = np.nextafter(v, 3) if sgn > 0 else np.nextafter(v, 0)
    return out


def fexact(N_, D_, yy):
    Y = yy.astype(L)
    n_ = np.zeros_like(Y)
    d_ = np.zeros_like(Y)
    for a in reversed(N_):
        n_ = n_ * Y + L(a)
    for a in reversed(D_):
        d_ = d_ * Y + L(a)
    return n_ / d_


for ch, k, c in specs:
    yl = side(c, -1)
    yr = side(c, 1)
    fl = fexact(N, D, yl)
    fr = fexact(N, D, yr)
    ci = 6 - k
    row = []
    for dlt in range(8):
        N2 = list(N)
        D2 = list(D)
        if ch == "N":
            N2[ci] = N2[ci] + dlt * np.spacing(N2[ci])
        else:
            D2[ci] = D2[ci] + dlt * np.spacing(D2[ci])
        ml = G(N2, D2, yl)
        mr = G(N2, D2, yr)
        row.append(
            float(
                np.mean((mr.astype(L) - fr) / L(np.spacing(np.abs(mr))))
                - np.mean((ml.astype(L) - fl) / L(np.spacing(np.abs(ml))))
            )
        )
    print("%s.S%d c=%.4f step(delta=0..7): %s" % (ch, k, c, " ".join("%+.3f" % v for v in row)), flush=True)

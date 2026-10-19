# Excel function = grid_a rational + smooth delta; delta/(x-2) as Legendre in 2(x-2.5)
import numpy as np, pickle
from bits import *


def build(deg=6, bw=0.01):
    X, Y = loadall()
    d = Y - sim(list(P2), list(Q2), X)
    b = np.floor((X - 2) / bw).astype(int)
    nb = np.bincount(b)
    ok = nb > 0
    md = np.bincount(b, d)[ok] / nb[ok]
    xc = np.bincount(b, X)[ok] / nb[ok]
    v = np.bincount(b, d * d)[ok] / nb[ok] - md**2
    se = np.sqrt(np.maximum(v, 1e-40) / nb[ok])
    g = md / (xc - 2)
    gse = se / (xc - 2)
    V = np.polynomial.legendre.legvander((xc - 2.5) * 2, deg)
    co = np.linalg.lstsq(V / gse[:, None], g / gse, rcond=None)[0]
    return co, np.sum(((V @ co - g) / gse) ** 2) / (len(g) - deg - 1)


if __name__ == "__main__":
    co, chi = build(6)
    print("chi2/dof", chi)
    pickle.dump(co, open("gcorr6.pkl", "wb"))

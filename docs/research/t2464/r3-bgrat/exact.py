"""T-2512 r3-bgrat: exact pieces for every recorded bgrat call (calls.pkl -> exact.pkl, both git-ignored).

Per call (a, b, x, y, w, lnx, g): z = fl(-nu lnx) as TOMS forms it in doubles (nu = a + 0.5 (b - 1)),
F = exact I at ln x = lnx (mpmath 40 digits), Qz = exact Q(b, z) at the double z."""
from __future__ import annotations

import pickle
import sys
from multiprocessing import Pool
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent


def zdouble(a: float, b: float, lnx: float) -> float:
    bm1 = b - 0.5 - 0.5
    nu = a + 0.5 * bm1
    return -nu * lnx


def work(k):
    a, b, lnx = k
    mpmath.mp.dps = 40
    L = mpmath.mpf(lnx)
    X = mpmath.exp(L)
    try:
        F = mpmath.betainc(a, b, 0, X, regularized=True)
    except Exception:  # noqa: BLE001
        F = 1 - mpmath.betainc(b, a, 0, -mpmath.expm1(L), regularized=True)
    z = zdouble(a, b, lnx)
    Q = mpmath.gammainc(b, z, mpmath.inf, regularized=True)
    return k, (F, Q)


if __name__ == "__main__":
    d = pickle.load(open(HERE / "calls.pkl", "rb"))
    keys = sorted({(c[0], c[1], c[5]) for c in d["calls"].values() if c[1] != 0.5 or "-a" in sys.argv})
    out = HERE / "exact.pkl"
    have = pickle.load(open(out, "rb")) if out.exists() else {}
    todo = [k for k in keys if k not in have]
    print(len(keys), "keys,", len(todo), "to do", flush=True)
    with Pool(8) as p:
        for i, (k, v) in enumerate(p.imap_unordered(work, todo, chunksize=8)):
            have[k] = v
            if i % 500 == 0:
                print(i, flush=True)
    pickle.dump(have, open(out, "wb"))

"""b11: is the x = 1 residual (Excel's E minus exact ln F(1), interval centre, units) a per-shape-parameter error?
Least-squares fits on the unordered pairs (the swap pair is bit-identical): free u(w) + v(o); free e(w) + e(o) shared
(the stirlerr-error pattern -e(w) - e(o), h ignored); free e on w, o and h together (h on its own grid).
Usage: b11_additive.py SET"""

from __future__ import annotations

import sys

import numpy as np

import b6_x1 as X
import b9_x1reg as R
import blib as B


def residuals(name):
    out = {}
    for r in B.load(name):
        if r["x"] != 1.0:
            continue
        w, o = min(r["d1"], r["d2"]), max(r["d1"], r["d2"])
        t = X.target(r)
        _, E = R.columns(r["d1"], r["d2"])
        out.setdefault((w, o), []).append(float(((t[0] + t[1]) / 2 - E) * B.U))
    return {k: float(np.mean(v)) for k, v in out.items()}


def fit(res, keyf):
    keys = sorted({k for p in res for k in keyf(p)})
    ix = {k: i for i, k in enumerate(keys)}
    M = np.zeros((len(res), len(keys) + 1))
    y = np.zeros(len(res))
    for j, (p, v) in enumerate(res.items()):
        for k, s in keyf(p).items():
            M[j, ix[k]] += s
        M[j, -1] = 1
        y[j] = v
    coef, *_ = np.linalg.lstsq(M, y, rcond=None)
    return float(np.sqrt(np.mean((y - M @ coef) ** 2))), len(keys) + 1


def _sum(*ds):
    out: dict = {}
    for d in ds:
        for k, v in d.items():
            out[k] = out.get(k, 0) + v
    return out


if __name__ == "__main__":
    res = residuals(sys.argv[1])
    print("pairs", len(res), "raw rms", round(float(np.sqrt(np.mean(np.array(list(res.values())) ** 2))), 3))
    for nm, kf in (
        ("u(w)+v(o)", lambda p: {("w", p[0]): 1, ("o", p[1]): 1}),
        ("e(w)+e(o)", lambda p: _sum({p[0] / 2: 1}, {p[1] / 2: 1})),
        ("e(w)+e(o)-e(h)", lambda p: _sum({p[0] / 2: 1}, {p[1] / 2: 1}, {(p[0] + p[1]) / 2: -1})),
        ("u(w)+v(o)+k(h)", lambda p: {("w", p[0]): 1, ("o", p[1]): 1, ("h", (p[0] + p[1]) / 2): 1}),
    ):
        rms, k = fit(res, kf)
        print(f"{nm:16s} params {k:4d} rms {rms:.3f}")


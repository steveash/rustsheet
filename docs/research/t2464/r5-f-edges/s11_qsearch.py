"""Brute-force search for the hidden addend Q of fl(Q + P) over stored sums of up to three signed Excel pieces.
Each candidate is scored by the number of clean-region groups (>= 2 grid sizes) whose D(q) residual spread is within
TOL units. --plant NAME replaces Excel's D by C - psi_q(candidate NAME) + uniform noise of +-1 unit (positive
control). Usage: s11_qsearch.py [--tol 3.5] [--plant NAME] [--top 25]"""

from __future__ import annotations

import itertools
import math
import random
import sys
from fractions import Fraction

import numpy as np

import qbits as QB

L, LDB, st = QB.L, QB.LDB, QB.st
TOL = float(sys.argv[sys.argv.index("--tol") + 1]) if "--tol" in sys.argv else 3.5
TOP = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 25
PLANT = sys.argv[sys.argv.index("--plant") + 1] if "--plant" in sys.argv else None
LN2PI = float(L.mpmath.log(2 * L.mpmath.pi))


def pieces(m: float, n: float, side: int) -> dict:
    a, b, h = m / 2, n / 2, (m + n) / 2
    w, o = (a, b) if side == 1 else (b, a)
    c = a if side == 1 else -b
    LN = lambda v: float(L.LN(v))  # noqa: E731
    mul = lambda k, v: st(LDB(k) * LDB(v))  # noqa: E731
    p = {
        "gh": L.GL(h),
        "ga": L.GL(a),
        "gb": L.GL(b),
        "LNm": LN(m),
        "LNn": LN(n),
        "LNmn": LN(st(LDB(m) / LDB(n))),
        "aLNm": mul(a, LN(m)),
        "bLNn": mul(b, LN(n)),
        "hLNn": mul(h, LN(n)),
        "hLNm": mul(h, LN(m)),
        "aLNn": mul(a, LN(n)),
        "bLNm": mul(b, LN(m)),
        "cLNmn": mul(c, LN(st(LDB(m) / LDB(n)))),
        "hLNh": mul(h, LN(h)),
        "aLNa": mul(a, LN(a)),
        "bLNb": mul(b, LN(b)),
        "hlnh-": mul(h - 0.5, LN(h)),
        "alna-": mul(a - 0.5, LN(a)),
        "blnb-": mul(b - 0.5, LN(b)),
        "lnsq2pi": LN2PI / 2,
        "A": st(LDB(h - 0.5) * L.L_xl(st(LDB(w) / LDB(o)))),
        "hLwo": mul(0.5, LN(w)),
        "sth": float(L.F.stirlerr_exact(h)),
        "sta": float(L.F.stirlerr_exact(a)),
        "stb": float(L.F.stirlerr_exact(b)),
        "h": h,
        "a": a,
        "b": b,
    }
    return p


def units_mod(v: float, M: int = 1024) -> float:
    """v (a double) in units of 2^-53, modulo M, exactly."""
    q = Fraction(v) * 2**53
    return float(q - M * math.floor(q / M))


def cand_values(p: dict):
    names = list(p)
    vals = {k: float(v) for k, v in p.items()}
    yield "0", 0.0
    for k in names:
        for s in (1, -1):
            yield f"{'+' if s > 0 else '-'}{k}", s * vals[k]
    for k1, k2 in itertools.permutations(names, 2):
        for s1, s2 in ((1, 1), (1, -1), (-1, 1), (-1, -1)):
            v = st(LDB(s1 * vals[k1]) + LDB(s2 * vals[k2]))
            yield f"{'+' if s1 > 0 else '-'}{k1}{'+' if s2 > 0 else '-'}{k2}", v


def spread_vec(Qm: np.ndarray, cells) -> np.ndarray:
    """Residual spread per candidate (Qm = Q in units mod 1024)."""
    res = []
    for c in cells:
        q = float(c["q"])
        t = Qm / q
        psi = (np.rint(t) - t) * q  # pred = round(Q/q) q - Q
        res.append(float(c["D"]) - psi)
    R = np.array(res)
    return R.max(axis=0) - R.min(axis=0)


def main():
    G = {k: v for k, v in QB.groups().items() if len({float(c["q"]) for c in v}) >= 2}
    score: dict = {}
    rng = random.Random(7)
    for k, cells in sorted(G.items()):
        p = pieces(*k)
        names, vals = zip(*cand_values(p))
        Qm = np.array([units_mod(v) for v in vals])
        if PLANT:
            j = names.index(PLANT)
            for c in cells:
                q = float(c["q"])
                t = Qm[j] / q
                c["D"] = 17.0 - (round(t) - t) * q + rng.uniform(-1, 1)
        sp = spread_vec(Qm, cells)
        for nm, s in zip(names, sp):
            score[nm] = score.get(nm, 0) + (s <= TOL)
    print(f"# {len(G)} groups, {len(score)} candidates, tol {TOL}" + (f", planted {PLANT}" if PLANT else ""))
    for nm, s in sorted(score.items(), key=lambda kv: -kv[1])[:TOP]:
        print(f"{s:3d}  {nm}")


if __name__ == "__main__":
    main()

"""T-2464 r2-tdist: shared data, compositions and scoring for the Student t family.

Data: ../cells.tsv rows for T.DIST.RT, T.DIST.2T, TDIST, T.DIST (TRUE and FALSE). Each row carries t, the truncated
df n, Excel's value, case/cell and the md5 hold-out flag (odd half, same rule as r2-bratio).
The incomplete-beta kernel is r2-bratio's `bratio_excel.kernel` (imported, not copied).
Scores: bit-exact / within 4 ulp / within half a unit of the 15th significant digit (w15), as in r2-bratio.
Run with /home/user/rustsheet/bindings/python/.venv/bin/python from this directory.
"""

from __future__ import annotations

import csv
import hashlib
import math
import pickle
import sys
from pathlib import Path

import mpmath

HERE = Path(__file__).resolve().parent
T2464 = HERE.parent
sys.path.insert(0, str(T2464 / "r2-bratio"))
sys.path.insert(0, str(T2464))
import bratio_excel as bx  # noqa: E402
import score_lib as sl  # noqa: E402

mpf = mpmath.mpf
TALLY, W15, ULPD = sl.tally, sl.w15, sl.ulpd
_EXF = HERE / "exact_cache.pkl"
_EX: dict = pickle.loads(_EXF.read_bytes()) if _EXF.exists() else {}


def hold(case: str, cell: str) -> bool:
    return int(hashlib.md5(repr((case, cell)).encode()).hexdigest(), 16) % 2 == 1


def load(kinds=("rt", "2t", "cdf", "pdf")):
    """Rows: kind ('rt' T.DIST.RT, '2t' T.DIST.2T / TDIST tails 2, 'rt1' TDIST tails 1, 'cdf', 'pdf'), t, n, excel."""
    rows = []
    for r in csv.DictReader(open(T2464 / "cells.tsv"), delimiter="\t"):
        fn = r["fn"]
        if fn not in ("T.DIST.RT", "T.DIST.2T", "TDIST", "T.DIST"):
            continue
        a = r["args"].split("|")
        try:
            t, df = float(a[0]), float(a[1])
            ex = float(r["excel"])
            tails = math.trunc(float(a[2])) if fn == "TDIST" else 0
        except ValueError:
            continue
        if not math.isfinite(ex):
            continue
        if fn == "T.DIST.RT":
            kind = "rt"
        elif fn == "T.DIST.2T":
            kind = "2t"
        elif fn == "TDIST":
            kind = "2t" if tails == 2 else "rt1"
        else:
            kind = "cdf" if a[2] == "True" else "pdf"
        if kind == "rt1":
            kind = "rt"
        if kind not in kinds:
            continue
        n = float(math.trunc(df))
        if n < 1:
            continue
        rows.append(
            dict(
                fn=fn,
                kind=kind,
                t=t,
                n=n,
                excel=ex,
                case=r["case"],
                cell=r["cell"],
                hold=hold(r["case"], r["cell"]),
            )
        )
    return rows


def targs(t: float, n: float):
    """The worksheet's rounded arguments (w5, statistics.md): tt = fl(t t), u = fl(n + tt), y = fl(n/u),
    z = fl(tt/u)."""
    tt = t * t
    u = n + tt
    return tt, u, n / u, tt / u


def exact_L(t: float, n: float) -> float:
    """P(T > |t|) at the double t, exact (mpmath, 50 digits): I_y(n/2, 1/2)/2 with y = n/(n + t^2) exact."""
    return float(_exact_L_mp(t, n))


def exact_pdf(t: float, n: float) -> float:
    k = ("pdf", abs(t), n)
    if k not in _EX:
        with mpmath.workdps(40):
            T, N = mpf(t), mpf(n)
            lg = mpmath.loggamma((N + 1) / 2) - mpmath.loggamma(N / 2)
            v = mpmath.exp(lg - (N + 1) / 2 * mpmath.log1p(T * T / N)) / mpmath.sqrt(N * mpmath.pi)
            _EX[k] = float(v)
    return _EX[k]


def exact_cell(r) -> float:
    if r["kind"] == "pdf":
        return exact_pdf(r["t"], r["n"])
    L = exact_L(r["t"], r["n"])
    with mpmath.workdps(40):
        if r["kind"] == "2t":
            return float(2 * mpf(L)) if r["t"] >= 0 else math.nan
        lower = r["kind"] == "cdf"
        # rt: P(T > t); cdf: P(T <= t)
        pos = r["t"] >= 0
        if lower == pos:
            return float(1 - mpf(_exact_L_mp(r["t"], r["n"])))
        return L


def _exact_L_mp(t: float, n: float):
    k = ("L2", abs(t), n)
    if k not in _EX:
        with mpmath.workdps(50):
            T, N = mpf(abs(t)), mpf(n)
            Y = N / (N + T * T)
            try:
                _EX[k] = mpmath.betainc(N / 2, mpf(0.5), 0, Y, regularized=True) / 2
            except (mpmath.libmp.NoConvergence, ValueError):  # y next to 1 at huge n: the complement does not cancel there
                _EX[k] = (1 - mpmath.betainc(mpf(0.5), N / 2, 0, 1 - Y, regularized=True)) / 2
    return _EX[k]


def save_exact():
    _EXF.write_bytes(pickle.dumps(_EX))


def line(name: str, pairs) -> str:
    n, ex, f, v = TALLY(pairs)
    return f"{name}: {ex}/{f}/{v} of {n}"

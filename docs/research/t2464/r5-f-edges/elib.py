"""T-2586 r5-f-edges: shared helpers for the edges of Excel's F density F.DIST(x, d1, d2, FALSE).
The value chain is r4-f-small-df's frozen M-sd (`sdmodel`, imported), wrapped here with Excel's argument rules
(df truncation and range, x <= 0, the d1*x overflow, under/overflow of the chain). Regions: `region(r)`."""

from __future__ import annotations

import importlib.util
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T / "r4-f-small-df",):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import sdmodel as SD  # noqa: E402
import slib as L  # noqa: E402

F = L.F
mpf = L.mpf


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def region(r) -> str:
    """Edge region of a row (first match): d1 = 1, d1 = 2, d2 = 1, d2 = 2, tiny x (< 1E-6), huge x (> 1E6), rest."""
    d1, d2, x = r["d1"], r["d2"], r["x"]
    if d1 == 1:
        return "d1=1"
    if d1 == 2:
        return "d1=2"
    if d2 == 1:
        return "d2=1"
    if d2 == 2:
        return "d2=2"
    if x < 1e-6:
        return "tinyx"
    if x > 1e6:
        return "hugex"
    return "rest"


REGIONS = ("d1=1", "d1=2", "d2=1", "d2=2", "tinyx", "hugex", "rest")


def report(name: str, rows, models: dict, by=region, keys=REGIONS) -> None:
    for k, fn in models.items():
        sc = L.score_rows(rows, fn, by)
        print(f"{name:10s} {k:8s} all {F.fmt(sc['all'])} | " + " | ".join(f"{c} {F.fmt(sc[c])}" for c in keys if c in sc))


# ------------------------------------------------------------------ mixed scoring (errors, zeros, numbers)
def _ulps(a: float, b: float) -> int:
    import struct

    ia, ib = (struct.unpack("<q", struct.pack("<d", v))[0] for v in (a, b))
    return abs(ia - ib)


def grade(model, excel) -> tuple[bool, bool, bool]:
    """(bit-exact, within 4 ulp, within 15 digits) of a model result against Excel's (float or error string)."""
    if isinstance(model, str) or isinstance(excel, str) or model == 0 or excel == 0:
        same = model == excel and type(model) is type(excel)
        return same, same, same
    u = _ulps(float(model), float(excel))
    return u == 0, u <= 4, bool(L.w15(float(model), float(excel)))


def tally(pairs) -> str:
    g = [grade(m, e) for m, e in pairs]
    return f"{sum(x[0] for x in g):5d}/{sum(x[1] for x in g):5d}/{sum(x[2] for x in g):5d} of {len(g):5d}"


# ------------------------------------------------------------------ Excel's admissible exponent (per cell)
def e_needed(x: float, excel: float, fin: str = "div", W: float = 4096.0):
    """(E0, lo, hi): the extended exponents E0 + u*2^-53, u in [lo, hi], that reproduce Excel through the last step
    (fin 'div': fl(EXP(E)/x) with EXP's result stored as a double; 'exp': EXP(E), E holding the -ln x), with
    E0 = ln(excel * x) | ln(excel). None if no u in [-W, W] fits."""
    R3 = L.R3
    LD = L.LDB
    E0 = L.ld(mpmath_log(excel) + (mpmath_log(x) if fin == "div" else 0))
    U1 = LD(2.0) ** -53
    X = LD(x)

    def fn(u):
        E = E0 + LD(u) * U1
        return L.st(LD(R3.exp_v6(E)) / X) if fin == "div" else R3.exp_v6(E)

    iv = R3.interval(fn, excel, W=W, tol=1e-3)
    return (E0, iv[0][0], iv[0][1]) if iv else None


def mpmath_log(v):
    return L.mpmath.log(L.mpf(v))

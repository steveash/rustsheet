"""verify-r4-f-small-df: independent parser + scorer for F.DIST(x,d1,d2,FALSE) probe cases.

Own code: case/oracle parsing (x from the exact two-piece formula, checked against Excel's echo in column A), exact
density (mpmath, 60 digits), tallies (bit-exact / within 4 ulp / within half a unit of the 15th significant digit).
Imported only: the worker's frozen model (sdmodel.value, read and checked against its description) and, for an
independent recomposition of M-sd (vm_sd below), the Excel primitives it is built from (Excel LN, EXP V6 on an
extended input, Excel log1p, x87 store) and r3-loader-const's grid_e.tsv.

Usage: vscore.py CASE_ID [CASE_ID ...]   (case ids under eval/suites/probes-numeric/cases)"""

from __future__ import annotations

import json
import math
import re
import struct
import sys
from decimal import Decimal
from fractions import Fraction
from pathlib import Path

import mpmath
import numpy as np

HERE = Path(__file__).resolve().parent
T = HERE.parent
CASES = T.parents[2] / "eval/suites/probes-numeric/cases"
sys.path.insert(0, str(T / "r4-f-small-df"))
mpmath.mp.dps = 60

_TERM = re.compile(r"(\d+)\*2\^(-?\d+)")
_XF = re.compile(r"^=\(?\d+\*2\^-?\d+(\+\d+\*2\^-?\d+)*\)?$")
_FD = re.compile(r"^=F\.DIST\(A(\d+),([0-9.]+),([0-9.]+),FALSE\)$")


def x_of(formula: str) -> float:
    m = _XF.match(formula)
    if not m:
        raise ValueError(formula)
    q = sum((Fraction(int(c)) * Fraction(2) ** int(e) for c, e in _TERM.findall(formula)), Fraction(0))
    v = float(q)
    if Fraction(v) != q:
        raise ValueError(f"not an exact double: {formula}")
    return v


def rows(case_id: str) -> list[dict]:
    case = json.loads((CASES / f"{case_id}.case.json").read_text())
    orc = json.loads((CASES / f"{case_id}.oracle.json").read_text())["sheets"]["Calc"]
    cells = case["sheets"][0]["cells"]
    out, bad_echo = [], 0
    for ref, f in cells.items():
        m = _FD.match(f) if ref.startswith("B") else None
        if not m:
            continue
        i, d1, d2 = int(m.group(1)), float(m.group(2)), float(m.group(3))
        x = x_of(cells[f"A{i}"])
        echo, ov = orc.get(f"A{i}"), orc.get(ref)
        if echo is None or echo.get("t") != "n" or echo["v"] != x:
            bad_echo += 1
            continue
        if ov is None or ov.get("t") != "n" or not ov["v"] > 0:
            continue
        out.append({"x": x, "d1": d1, "d2": d2, "excel": float(ov["v"]), "row": i})
    if bad_echo:
        print(f"# {case_id}: {bad_echo} echo mismatches dropped")
    return out


def exact(x: float, m: float, n: float) -> mpmath.mpf:
    X, M, N = mpmath.mpf(x), mpmath.mpf(m), mpmath.mpf(n)
    lb = mpmath.loggamma(M / 2) + mpmath.loggamma(N / 2) - mpmath.loggamma((M + N) / 2)
    ln = (
        (M / 2) * mpmath.log(M / N)
        + (M / 2 - 1) * mpmath.log(X)
        - ((M + N) / 2) * mpmath.log1p(M * X / N)
        - lb
    )
    return mpmath.exp(ln)


def _bits(v: float) -> int:
    return struct.unpack("<q", struct.pack("<d", v))[0]


def judge(model: float, xl: float) -> tuple[bool, bool, bool]:
    if not math.isfinite(model):
        return False, False, False
    be = model == xl
    w4 = model > 0 and abs(_bits(model) - _bits(xl)) <= 4
    dx, dm = Decimal(xl), Decimal(model)
    half = Decimal(5) * Decimal(10) ** (dx.adjusted() - 15)
    return be, w4, abs(dx - dm) <= half


# ------------------------------------------------------------------ models
import sdmodel as SD  # noqa: E402  (the worker's frozen candidate)
import slib as WL  # noqa: E402  (only its Excel primitives are used by vm_sd)

LDB = np.longdouble
_GRID = {
    float(a): float(b)
    for a, b in (ln.split("\t") for ln in (T / "r3-loader-const/grid_e.tsv").read_text().splitlines()[1:])
}
_VS = [v for v in sorted(_GRID) if v >= 14]
_SLOPE, _ICPT = np.polyfit(_VS, [_GRID[v] for v in _VS], 1)


def _e(v: float) -> float:
    if v == 0.5:
        return 0.22
    if v == 1.0:
        return 0.06
    return _GRID[v] if v in _GRID else float(_SLOPE * v + _ICPT)


def _ld(q: mpmath.mpf) -> LDB:
    """Nearest long double of an mpf (via a 30-digit decimal string)."""
    return LDB(mpmath.nstr(q, 30, strip_zeros=False))


_KCACHE: dict = {}


def vm_K(m: float, n: float, side: int) -> LDB:
    """My recomposition of M-sd's constant from the RESULTS.md text: exact K + delta units."""
    key = (m, n, side)
    if key not in _KCACHE:
        a, b, h = mpmath.mpf(m) / 2, mpmath.mpf(n) / 2, mpmath.mpf(m + n) / 2
        lb = mpmath.loggamma(a) + mpmath.loggamma(b) - mpmath.loggamma(h)
        K = -lb + (a if side == 1 else -b) * mpmath.log(mpmath.mpf(m) / n)
        w, o = (m / 2, n / 2) if side == 1 else (n / 2, m / 2)
        q = WL.st(LDB(w) / LDB(o))
        lp = LDB((m + n) / 2 - 0.5) * WL.L_xl(q)
        p, qq = lp.as_integer_ratio()
        side_err = mpmath.mpf(p) / qq - (h - mpmath.mpf(1) / 2) * mpmath.log(h / o)
        delta = _e(float(h)) - _e(m / 2) - _e(n / 2) + 5.33 + float(side_err * mpmath.mpf(2) ** 53)
        _KCACHE[key] = _ld(K) + LDB(delta) * LDB(2.0) ** -53
    return _KCACHE[key]


def vm_sd(x: float, m: float, n: float, fin: str = "div") -> float:
    mx = WL.st(LDB(m) * LDB(x))
    side = 1 if mx <= n else 2
    r = WL.st(LDB(mx) / LDB(n)) if side == 1 else WL.st(LDB(n) / LDB(mx))
    c = m / 2 if side == 1 else -n / 2
    E = vm_K(m, n, side) + LDB(WL.st(LDB(c) * WL.LN(x))) - LDB((m + n) / 2) * WL.L_xl(r)
    ex = LDB(WL.EXP(E))
    return float(WL.st(ex / LDB(x)) if fin == "div" else WL.st(ex * LDB(WL.st(LDB(1) / LDB(x)))))


MODELS = {
    "CR": lambda r: float(r["exact"]),
    "M-xd": lambda r: float(WL.mxd(r)),
    "M-sd": lambda r: float(SD.value(r["x"], r["d1"], r["d2"])),
    "M-sdm": lambda r: float(SD.value(r["x"], r["d1"], r["d2"], "mulr")),
    "vM-sd": lambda r: vm_sd(r["x"], r["d1"], r["d2"]),
}


def cls(r) -> str:
    d1, d2 = r["d1"], r["d2"]
    if max(d1, d2) >= 50:
        return "max>=50"
    if min(d1, d2) <= 2:
        return "min<=2"
    return "both<=10" if max(d1, d2) <= 10 else "other"


def main(ids: list[str]) -> None:
    for cid in ids:
        rs = rows(cid)
        for r in rs:
            r["exact"] = exact(r["x"], r["d1"], r["d2"])
        print(f"# {cid}: n = {len(rs)}")
        agree = 0
        for name, fn in MODELS.items():
            tal: dict = {}
            for r in rs:
                v = fn(r)
                if name == "vM-sd":
                    agree += v == float(SD.value(r["x"], r["d1"], r["d2"]))
                for k in ("all", cls(r)):
                    t = tal.setdefault(k, [0, 0, 0, 0])
                    t[3] += 1
                    for j, ok in enumerate(judge(v, r["excel"])):
                        t[j] += ok
            print(
                f"{name:6s} "
                + " | ".join(f"{k} {t[0]}/{t[1]}/{t[2]} of {t[3]}" for k, t in sorted(tal.items()))
            )
        print(f"vM-sd == M-sd on {agree} of {len(rs)}")


if __name__ == "__main__":
    main(sys.argv[1:])

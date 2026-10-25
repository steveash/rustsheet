"""verify-r3-fdensity: the verifier's own case-formula parser, exact F density, scorer and data loaders.
Nothing here is imported from r3-fdensity except the candidate function (`model`, checked in vcheck.py).
Excel values come from the probes-numeric oracle sidecars; arguments are parsed from the case formulas, with
decimal literals truncated (not rounded) to 15 significant digits as Excel's formula parser does."""

from __future__ import annotations

import json
import math
import pickle
import re
import struct
import sys
from decimal import Decimal, getcontext
from fractions import Fraction
from pathlib import Path

import mpmath

HERE = Path(__file__).parent
T = HERE.parent
CASES = Path("/home/user/rustsheet/eval/suites/probes-numeric/cases")
mpmath.mp.prec = 200
getcontext().prec = 60
mpf = mpmath.mpf

# ------------------------------------------------------------------ formula parsing
_NUM = r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"
_ENC = re.compile(r"^=?\((\d+)\*2\^(-?\d+)\+(\d+)\*2\^(-?\d+)\)$")
_ENC1 = re.compile(r"^=?(\d+)\*2\^(-?\d+)$")
_FDIST = re.compile(rf"^=F\.DIST\(\s*([^,]+?)\s*,\s*({_NUM})\s*,\s*({_NUM})\s*,\s*(FALSE|0)\s*\)$", re.I)


def lit15(s: str) -> float:
    """A decimal literal as Excel reads it: truncated to 15 significant digits, then the nearest double."""
    d = Decimal(s)
    if d == 0:
        return 0.0
    sign, digits, exp = d.as_tuple()
    if len(digits) > 15:
        cut = len(digits) - 15
        digits, exp = digits[:15], exp + cut
    return float(Fraction(Decimal((sign, digits, exp))))


def enc_value(f: str) -> float | None:
    """Value of an exact two-piece encoder formula (hi*2^a+lo*2^b), or None."""
    m = _ENC.match(f.strip())
    m1 = _ENC1.match(f.strip())
    if m1:
        hi, ea, lo, eb = int(m1.group(1)), int(m1.group(2)), 0, 0
        if hi >= 10**15:  # a longer integer literal would be truncated by Excel
            raise ValueError(f"literal over 15 digits: {f}")
    elif not m:
        return None
    else:
        hi, ea, lo, eb = (int(g) for g in m.groups())
    v = Fraction(hi) * Fraction(2) ** ea + Fraction(lo) * Fraction(2) ** eb
    x = float(v)
    if Fraction(x) != v:
        raise ValueError(f"encoder value not a double: {f}")
    return x


def parse_fdist(f: str):
    """(x-token, d1, d2) of =F.DIST(x, d1, d2, FALSE) with literal df; x-token is a str (ref or literal)."""
    m = _FDIST.match(f.strip())
    if not m:
        return None
    return m.group(1), lit15(m.group(2)), lit15(m.group(3))


def resolve_x(tok: str, cells: dict, sheet: dict):
    """x of an F.DIST call: a literal, or a reference to an encoder cell (value checked against Excel's echo)."""
    if re.fullmatch(_NUM, tok):
        return lit15(tok), None
    if re.fullmatch(r"[A-Z]+\d+", tok):
        x = enc_value(cells.get(tok, ""))
        if x is None:
            return None, None
        echo = sheet.get(tok)
        ev = float(echo["v"]) if echo and echo.get("t") == "n" else None
        return x, ev
    return None, None


def load_case(cid: str):
    case = json.loads((CASES / f"{cid}.case.json").read_text())
    side = json.loads((CASES / f"{cid}.oracle.json").read_text())
    return case["sheets"][0]["cells"], side["sheets"]["Calc"]


def case_rows(cid: str, stats: dict | None = None):
    """All F.DIST(.,FALSE) cells of a case: dict(case, cell, x, d1, d2 (truncated to integers as Excel does), excel)."""
    cells, sheet = load_case(cid)
    st = stats if stats is not None else {}
    out = []
    for ref, f in cells.items():
        if not isinstance(f, str):
            continue
        p = parse_fdist(f)
        if p is None:
            if "F.DIST(" in f.upper() and "FALSE" in f.upper():
                st["unparsed"] = st.get("unparsed", 0) + 1
            continue
        x, echo = resolve_x(p[0], cells, sheet)
        if x is None:
            st["unparsed"] = st.get("unparsed", 0) + 1
            continue
        if echo is not None and echo != x:
            st["echo_mismatch"] = st.get("echo_mismatch", 0) + 1
            continue
        c = sheet.get(ref)
        if not c or c.get("t") != "n":
            st["non_numeric"] = st.get("non_numeric", 0) + 1
            continue
        ex = float(c["v"])
        d1, d2 = float(math.trunc(p[1])), float(math.trunc(p[2]))
        if not (x > 0 and d1 >= 1 and d2 >= 1 and ex > 0 and math.isfinite(ex)):
            st["outside"] = st.get("outside", 0) + 1
            continue
        out.append(dict(case=cid, cell=ref, x=x, d1=d1, d2=d2, excel=ex))
    return out


# ------------------------------------------------------------------ exact reference (own)
_PC = HERE / "cache.pkl"
_C: dict = pickle.loads(_PC.read_bytes()) if _PC.exists() else {}


def f_exact(x: float, d1: float, d2: float):
    k = ("F", x, d1, d2)
    if k not in _C:
        X, m, n = mpf(x), mpf(d1), mpf(d2)
        a, b = m / 2, n / 2
        lv = (mpmath.loggamma(a + b) - mpmath.loggamma(a) - mpmath.loggamma(b) + a * mpmath.log(m / n)
              + (a - 1) * mpmath.log(X) - (a + b) * mpmath.log1p(m * X / n))
        _C[k] = mpmath.exp(lv)
    return _C[k]


def save():
    _PC.write_bytes(pickle.dumps(_C))


# ------------------------------------------------------------------ scoring (own)
def _ord(v: float) -> int:
    i = struct.unpack("<q", struct.pack("<d", v))[0]
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def classify(model: float, excel: float):
    """(bit-exact, within 4 ulp, within half a unit of excel's 15th significant digit)."""
    if not math.isfinite(model):
        return False, False, False
    be = model == excel
    w4 = abs(_ord(model) - _ord(excel)) <= 4
    de = Decimal(excel)
    unit = Decimal(10) ** (de.adjusted() - 14)
    w15 = abs(Decimal(model) - de) * 2 <= unit
    return be, w4, w15


def tally(pairs):
    n = be = w4 = w15 = 0
    for m, e in pairs:
        a, b, c = classify(m, e)
        n += 1
        be += a
        w4 += b
        w15 += c
    return n, be, w4, w15


def fmt(t) -> str:
    n, be, w4, w15 = t
    return f"{be:4d}/{w4:5d}/{w15:5d} of {n:5d}"


# ------------------------------------------------------------------ the worker's candidate (import only)
def model(name: str):
    sys.path.insert(0, str(T / "r3-fdensity"))
    import c1_model as C  # noqa: E402

    o = C.VARIANTS[name]
    return lambda x, m, n: float(C.value(x, m, n, o))

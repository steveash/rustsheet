"""verify-r3-binom-pmf: an independent parser, exact reference and scorer for Excel's BINOM.DIST / BETA.DIST /
NEGBINOM.DIST densities (T-2513, verifier of r3-binom-pmf).  Nothing here is copied from r3-binom-pmf/blib.py.

- Formulas are parsed from the case files themselves (`=FN(a,b,c,FALSE)`), each numeric argument evaluated as
  Excel would: a decimal literal is truncated to 15 significant digits and then rounded to the nearest double;
  the exact-double form `(a*2^-e+b*2^-f)` is evaluated with Fractions and must be one double.
- Exact values: mpmath at 200 bits from the exact rational p.
- Scoring: exact / within 4 ulp / within 15 significant digits (half a unit of the 15th digit of Excel's value).
"""

from __future__ import annotations

import json
import math
import re
from decimal import ROUND_DOWN, Decimal
from fractions import Fraction
from pathlib import Path

import mpmath

mpmath.mp.prec = 200
ROOT = Path("/home/user/rustsheet")
SUITES = ROOT / "eval" / "suites"
CASES = SUITES / "probes-numeric" / "cases"

CALL = re.compile(r"^=(BINOM\.DIST|BINOMDIST|NEGBINOM\.DIST|NEGBINOMDIST|BETA\.DIST)\((.*)\)$")
DEC = re.compile(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$")
POW2 = re.compile(r"^\((\d+)\*2\^(-?\d+)\+(\d+)\*2\^(-?\d+)\)$")
POW1 = re.compile(r"^(\d+)\*2\^(-?\d+)$")


def excel_literal(s: str) -> Fraction | None:
    """Exact value Excel gives a numeric argument text, or None if it is not a plain literal."""
    s = s.strip()
    m = POW2.match(s)
    if m:
        a, e, b, f = (int(g) for g in m.groups())
        v = Fraction(a) * Fraction(2) ** e + Fraction(b) * Fraction(2) ** f
        if Fraction(float(v)) != v:
            raise ValueError(f"not one double: {s}")
        return v
    m = POW1.match(s)
    if m:
        a, e = (int(g) for g in m.groups())
        if len(m.group(1).lstrip("0")) > 15:
            raise ValueError(f"mantissa literal beyond 15 digits: {s}")
        v = Fraction(a) * Fraction(2) ** e
        if Fraction(float(v)) != v:
            raise ValueError(f"not one double: {s}")
        return v
    if not DEC.match(s):
        return None
    d = Decimal(s)
    if d != 0:  # Excel keeps 15 significant digits of a typed literal, truncating the rest
        q = Decimal(1).scaleb(d.adjusted() - 14)
        d = d.quantize(q, rounding=ROUND_DOWN)
    return Fraction(float(d))  # nearest double


def parse_cell(formula: str):
    """(kernel, k, n, p) for a pmf/pdf formula, else None.  kernel BIN: BINOM(k,n,p); NEG: NEGBINOM(f=k, r=n);
    BETA: BETA.DIST(p, k+1, n-k+1, FALSE) with integer shapes and A=0, B=1 (k, n reported as the binomial's)."""
    m = CALL.match(formula.strip())
    if not m:
        return None
    fn, body = m.groups()
    args = [a.strip() for a in body.split(",")]
    cumtxt = args[-1].upper()
    if fn == "NEGBINOMDIST":
        if len(args) != 3:
            return None
        vals = args
    else:
        if cumtxt not in ("FALSE", "0") or len(args) not in (4, 6 if fn == "BETA.DIST" else 4):
            return None
        vals = args[:3]
        if fn == "BETA.DIST" and len(args) == 6:
            return None  # A, B bounds: not in these probes
    try:
        v = [excel_literal(a) for a in vals]
    except ValueError:
        return None
    if None in v:
        return None
    if fn == "BETA.DIST":
        p, a, b = v
        if a.denominator != 1 or b.denominator != 1 or a < 1 or b < 1:
            return None
        return ("BETA", int(a) - 1, int(a + b) - 2, p)
    x, y, p = v
    if fn.startswith("NEG"):
        x, y = int(x), int(y)  # Excel truncates f and r
        return ("NEG", x, y, p)
    x, y = int(x), int(y)
    if not 0 <= x <= y:
        return None
    return ("BIN", x, y, p)


def load_case(path: Path):
    """Rows (case id, cell, kernel, k, n, p Fraction, excel float) of every parsed pmf cell with a numeric value."""
    spec = json.loads(path.read_text())
    side = path.with_name(path.name.replace(".case.json", ".oracle.json"))
    if not side.exists():
        return []
    vals = json.loads(side.read_text())["sheets"]
    out = []
    for sh in spec.get("sheets", []):
        got = vals.get(sh["name"], {})
        for cell, f in sh.get("cells", {}).items():
            if not isinstance(f, str):
                continue
            r = parse_cell(f)
            c = got.get(cell)
            if r is None or not c or c.get("t") != "n":
                continue
            kern, k, n, p = r
            if not 0 < p < 1:
                continue
            out.append(dict(case=path.name, cell=cell, kern=kern, k=k, n=n, p=p, excel=float(c["v"])))
    return out


def echo_misreads(path: Path):
    """Cells `=<literal>` whose Excel value is not the literal's double."""
    spec = json.loads(path.read_text())
    vals = json.loads(path.with_name(path.name.replace(".case.json", ".oracle.json")).read_text())["sheets"]
    bad, n = [], 0
    for sh in spec["sheets"]:
        for cell, f in sh["cells"].items():
            if isinstance(f, str) and f.startswith("=(") and "DIST" not in f:
                n += 1
                want = float(excel_literal(f[1:]))
                c = vals[sh["name"]].get(cell)
                if not c or c.get("t") != "n" or float(c["v"]) != want:
                    bad.append((cell, want, c))
    return n, bad


# ---------------------------------------------------------------- exact
_EX: dict = {}


def exact(kern: str, k: int, n: int, p: Fraction) -> mpmath.mpf:
    """Exact density.  BIN: C(n,k) p^k q^(n-k).  BETA: (n+1) times that.  NEG (f=k, r=n): C(f+r-1,f) p^r q^f."""
    key = (kern, k, n, p)
    if key in _EX:
        return _EX[key]
    P = mpmath.mpf(p.numerator) / p.denominator
    Q = mpmath.mpf((1 - p).numerator) / (1 - p).denominator
    if kern == "NEG":
        top, low, ea, eb = k + n - 1, k, n, k
    else:
        top, low, ea, eb = n, k, k, n - k
    lc = mpmath.loggamma(top + 1) - mpmath.loggamma(low + 1) - mpmath.loggamma(top - low + 1)
    v = mpmath.exp(lc + ea * mpmath.log(P) + eb * mpmath.log(Q))
    if kern == "BETA":
        v *= n + 1
    _EX[key] = v
    return v


# ---------------------------------------------------------------- scoring
def w15(m: float, e: float) -> bool:
    if m == e:
        return True
    if m is None or not math.isfinite(m) or e == 0:
        return False
    return abs(Fraction(m) - Fraction(e)) <= Fraction(5) * Fraction(10) ** (
        math.floor(math.log10(abs(e))) - 15
    )


def score(pairs) -> tuple[int, int, int, int]:
    ex = w4 = c15 = 0
    n = 0
    for m, e in pairs:
        n += 1
        if m is None or not math.isfinite(m):
            continue
        ex += m == e
        w4 += abs(m - e) <= 4 * math.ulp(e)
        c15 += w15(m, e)
    return ex, w4, c15, n


def fmt(t) -> str:
    return f"{t[0]}/{t[1]}/{t[2]} of {t[3]}"


def cr(r) -> float:
    return float(exact(r["kern"], r["k"], r["n"], r["p"]))

"""T-2586 r4-f-brcomp: Excel's F density as TOMS 708 brcomp at F's beta arguments, divided by x.

Hypothesis: F.DIST(x, d1, d2, FALSE) = brcomp(a, b, y, q) / x with a = d1/2, b = d2/2, y = 1/(1 + d2/(d1 x)),
q = 1/(1 + d1 x/d2) (w5-triangulation's F cdf arguments), with bratio's swap, using Excel's EXP/LN/GAMMALN.
Everything Excel-specific is imported: r2-bratio's Kernel (TOMS 708 brcomp/brcmp1, bcorr, rlog1, ...), r3-fdensity's
loaders/scorer (flib) and probe rows, r3-tdensity's Excel log1p (tdlib.log1p_xl).
Variant knobs (dict `o`), all default to plain TOMS with Excel's elementary functions:
  args   'w5' (above) | 'div' (y = mx/(mx+n), q = n/(mx+n)) | 'w5y' (q = 1 - y) | 'rs' (r = the smaller of
         n/mx, mx/n; big side 1/(1+r), small side r/(1+r)) | 'rsm' (small side r * fl(1/(1+r)))
  swap   'bratio' (bratio's lambda / x > 0.5 rule) | 'none' | 'always' | 'half' (swap when y > 1/2)
  alnrel 'toms' (TOMS's rational) | 'xl' (Excel's log1p: 14-term series to 0.375, LN(fl(1+a)) above)
  rlog1  'toms' | 'exact' (correctly rounded x - ln(1+x))
  fin    'div' fl(brcomp / x) | 'mul' fl(brcomp * fl(1/x))
  gamln  'excel' (Excel's GAMMALN wherever TOMS calls gamln/gsumln) | 'toms' (TOMS's own gamln)
"""

from __future__ import annotations

import importlib.util
import math
import sys
from dataclasses import replace
from pathlib import Path

HERE = Path(__file__).resolve().parent
T = HERE.parent
for p in (T / "r2-bratio", T / "r3-fdensity"):
    if str(p) not in sys.path:
        sys.path.insert(0, str(p))

import mpmath  # noqa: E402

import bratio_excel as BX  # noqa: E402  (r2-bratio: Kernel, Cfg, Excel EXP/LN/GAMMALN)
import flib as F  # noqa: E402  (r3-fdensity: loaders, exact F, scorer)


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


R3P = _load("r3_mkprobe", T / "r3-fdensity" / "mkprobe.py")  # r3's probes (a, sw, hold, holdsw)
R4P = _load("r4_mkprobe", HERE / "mkprobe.py")  # this directory's probes (hold, big)
TOMS_XL = BX.TOMS_XL  # plain TOMS 708 with Excel's EXP/LN/GAMMALN


class FKernel(BX.Kernel):
    """r2-bratio's Kernel with the knobs above on alnrel / rlog1."""

    def __init__(self, cfg, o: dict):
        super().__init__(cfg)
        self.o = o

    def alnrel(self, a: float) -> float:
        if self.o.get("alnrel", "toms") == "xl":
            return float(F.TD.log1p_xl(a))
        return super().alnrel(a)

    def rlog1(self, x: float) -> float:
        if self.o.get("rlog1", "toms") == "exact":
            X = mpmath.mpf(x)
            return float(X - mpmath.log1p(X))
        return super().rlog1(x)


_K: dict = {}


def kern(o: dict) -> FKernel:
    key = tuple(sorted(o.items()))
    if key not in _K:
        _K[key] = FKernel(replace(TOMS_XL, gamln=o.get("gamln", "excel")), o)
    return _K[key]


def args(x: float, m: float, n: float, form: str):
    mx = m * x
    if form in ("rs", "rsm"):  # smaller ratio r, s = 1 + r: big side 1/s, small side r/s ('rs') or r*(1/s) ('rsm')
        r = n / mx if mx > n else mx / n
        s = 1.0 + r
        big = 1.0 / s
        small = r / s if form == "rs" else r * big
        return (big, small) if mx > n else (small, big)
    if form.startswith("w5"):
        y = 1.0 / (1.0 + n / mx)
        q = 1.0 / (1.0 + mx / n)
    else:
        den = n + mx
        y, q = mx / den, n / den
    if form.endswith("y"):
        q = 1.0 - y
    return y, q


def swapped(a: float, b: float, y: float, q: float, rule: str) -> bool:
    if rule == "none":
        return False
    if rule == "always":
        return True
    if rule == "half":
        return y > 0.5
    if min(a, b) <= 1.0:  # bratio: swap when x > 0.5
        return y > 0.5
    lam = (a + b) * q - b if a > b else a - (a + b) * y
    return lam < 0.0


def model(x: float, m: float, n: float, o: dict | None = None) -> float:
    o = o or {}
    a, b = m / 2.0, n / 2.0
    y, q = args(x, m, n, o.get("args", "w5"))
    if swapped(a, b, y, q, o.get("swap", "bratio")):
        a, b, y, q = b, a, q, y
    k = kern(o)
    bc = k.brcmp1(0, a, b, y, q, toms=True)
    if o.get("fin", "div") == "mul":
        return bc * (1.0 / x)
    return bc / x


# ------------------------------------------------------------------ data
def load(name: str):
    """Rows with Excel's value and the exact density: r3's 'a', 'sw', 'hold', 'holdsw', 'cells', the verifier's
    'vhold' is not loaded here; this directory's 'big' and 'hold4'."""
    if name == "cells":
        rows = F.cells_rows()
    elif name in ("big", "hold4"):
        rows = R4P.rows("big" if name == "big" else "hold")
    else:
        rows = R3P.rows(name)
    rows = F.with_exact(rows)
    BX.GAMMALN(1.0)
    return rows


def prefetch_gl(rows, o_list):
    """Run every model once, then fetch the GAMMALN values the kernel asked for and are not cached."""
    for o in o_list:
        for r in rows:
            model(r["x"], r["d1"], r["d2"], o)
    return BX.fetch_missing()


def cls(r) -> str:
    d1, d2 = r["d1"], r["d2"]
    if max(d1, d2) >= 50:
        return "max>=50"
    if min(d1, d2) <= 2:
        return "min<=2"
    return "other"


def score(rows, fn):
    """{class: tally} with an 'all' key; fn(row) -> model value."""
    out: dict = {}
    for r in rows:
        v = fn(r)
        for c in ("all", cls(r)):
            out.setdefault(c, []).append((v, r["excel"]))
    return {c: F.tally(p) for c, p in out.items()}


def line(label: str, s: dict) -> str:
    parts = [f"{label:28s} all {F.fmt(s['all'])}"]
    for c in ("max>=50", "min<=2", "other"):
        if c in s:
            parts.append(f"{c} {F.fmt(s[c])}")
    return " | ".join(parts)

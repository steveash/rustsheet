"""Verifier for T-2513 r3-tdensity: independent case parser, scorer and correctly-rounded reference.

* Cases are read straight from eval/suites/*/cases/*.case.json; values from the .oracle.json sidecars.
* Formula arguments are parsed by a small recursive-descent evaluator (numbers, cell refs, + - * / ^, unary minus,
  parentheses).  Excel truncates numeric literals in formulas to 15 significant digits (not rounding); numbers stored
  as JSON values in cells are kept as the double they are.
* The density cell must be exactly `=T.DIST(arg, arg, FALSE)`; df is truncated to an integer (Excel).
Nothing here is imported from the worker.
"""

from __future__ import annotations

import glob
import json
import math
import re
from decimal import ROUND_DOWN, Decimal, getcontext
from pathlib import Path

import mpmath

getcontext().prec = 60
SUITES = Path("/home/user/rustsheet/eval/suites")
TOP = re.compile(r"^=T\.DIST\((.*),\s*FALSE\s*\)$", re.I)


# ---------------------------------------------------------------- formula evaluation
def lit15(s: str) -> float:
    """Excel's reading of a numeric literal in a formula: truncate to 15 significant digits."""
    d = Decimal(s)
    if d == 0:
        return 0.0
    e = d.adjusted()
    q = Decimal(1).scaleb(e - 14)
    return float(d.quantize(q, rounding=ROUND_DOWN))


class Ev:
    TOK = re.compile(r"\s*(\d+\.?\d*(?:[eE][+-]?\d+)?|\.\d+(?:[eE][+-]?\d+)?|\$?[A-Z]{1,3}\$?\d+|[-+*/^()])")

    def __init__(self, cells: dict, depth: int = 0):
        self.cells, self.depth = cells, depth

    def __call__(self, s: str) -> float:
        self.toks, pos = [], 0
        s = s.strip()
        while pos < len(s):
            m = self.TOK.match(s, pos)
            if not m:
                raise ValueError(f"bad token in {s!r} at {pos}")
            self.toks.append(m.group(1))
            pos = m.end()
        self.i = 0
        v = self.add()
        if self.i != len(self.toks):
            raise ValueError(f"trailing tokens in {s!r}")
        return v

    def peek(self):
        return self.toks[self.i] if self.i < len(self.toks) else None

    def take(self):
        t = self.toks[self.i]
        self.i += 1
        return t

    def add(self):
        v = self.mul()
        while self.peek() in ("+", "-"):
            op = self.take()
            r = self.mul()
            v = v + r if op == "+" else v - r
        return v

    def mul(self):
        v = self.pow()
        while self.peek() in ("*", "/"):
            op = self.take()
            r = self.pow()
            v = v * r if op == "*" else v / r
        return v

    def pow(self):  # Excel: unary minus binds tighter than ^, ^ is left-associative
        v = self.unary()
        while self.peek() == "^":
            self.take()
            v = v ** self.unary()
        return v

    def unary(self):
        if self.peek() == "-":
            self.take()
            return -self.unary()
        if self.peek() == "+":
            self.take()
            return self.unary()
        return self.atom()

    def atom(self):
        t = self.take()
        if t == "(":
            v = self.add()
            if self.take() != ")":
                raise ValueError("paren")
            return v
        if t[0].isdigit() or t[0] == ".":
            return lit15(t)
        ref = t.replace("$", "")
        return cell_value(self.cells, ref, self.depth + 1)


def cell_value(cells: dict, ref: str, depth: int = 0) -> float:
    if depth > 20:
        raise ValueError("depth")
    v = cells[ref]
    if isinstance(v, bool):
        raise ValueError("bool")
    if isinstance(v, (int, float)):
        return float(v)
    if isinstance(v, str) and v.startswith("="):
        return Ev(cells, depth)(v[1:])
    return float(v)


def split_args(s: str) -> list[str]:
    out, d, cur = [], 0, ""
    for ch in s:
        if ch == "(":
            d += 1
        elif ch == ")":
            d -= 1
        if ch == "," and d == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return out


def load_case(path: str):
    """Rows {case, cell, t, df_raw, n, excel, tref, t_readback} for every =T.DIST(a, b, FALSE) cell with a numeric
    oracle value.  t_readback is the oracle's value of a referenced t cell (None if t is not a plain ref)."""
    p = Path(path)
    side = p.with_name(p.name.replace(".case.json", ".oracle.json"))
    if not side.exists():
        return [], 0
    case = json.loads(p.read_text())
    orc = json.loads(side.read_text())["sheets"]
    rows, bad = [], 0
    for sh in case["sheets"]:
        cells = sh["cells"]
        osh = orc.get(sh["name"], {})
        for ref, f in cells.items():
            if not isinstance(f, str):
                continue
            m = TOP.match(f.strip())
            if not m:
                continue
            a = split_args(m.group(1))
            if len(a) != 2:
                continue
            o = osh.get(ref)
            if o is None or o.get("t") != "n":
                continue
            try:
                t = Ev(cells)(a[0])
                df = Ev(cells)(a[1])
            except (ValueError, KeyError, ZeroDivisionError, OverflowError):
                bad += 1
                continue
            rb = None
            ra = a[0].strip().replace("$", "")
            if re.fullmatch(r"[A-Z]{1,3}\d+", ra) and isinstance(cells.get(ra), str):
                oc = osh.get(ra)
                rb = float(oc["v"]) if oc and oc.get("t") == "n" else None
            rows.append(
                dict(
                    case=p.name.replace(".case.json", ""),
                    cell=ref,
                    t=t,
                    df_raw=df,
                    n=float(math.trunc(df)),
                    excel=float(o["v"]),
                    t_readback=rb,
                )
            )
    return rows, bad


def load_all():
    rows, bad = [], 0
    for f in sorted(glob.glob(str(SUITES / "*" / "cases" / "*.case.json"))):
        r, b = load_case(f)
        rows += r
        bad += b
    return rows, bad


# ---------------------------------------------------------------- scoring
def w15(m: float, e: float) -> bool:
    """|m - e| <= half a unit in the 15th significant digit of e (exact decimal arithmetic)."""
    if m == e:
        return True
    if not (math.isfinite(m) and math.isfinite(e)) or e == 0:
        return False
    de, dm = Decimal(e), Decimal(m)
    return abs(dm - de) <= Decimal(5).scaleb(de.copy_abs().adjusted() - 15)


def ulps(m: float, e: float) -> float:
    return (m - e) / math.ulp(e) if e != 0 else math.inf


def score(pairs) -> tuple:
    n = ex = u4 = v15 = 0
    for m, e in pairs:
        n += 1
        ex += m == e
        u4 += abs(m - e) <= 4 * math.ulp(e)
        v15 += w15(m, e)
    return n, ex, u4, v15


def fmt(s) -> str:
    return f"{s[1]}/{s[2]}/{s[3]} of {s[0]}"


def cr(t: float, n: float) -> float:
    """Correctly rounded Student t density at the double t (mpmath, 50 digits)."""
    with mpmath.workdps(50):
        T, N = mpmath.mpf(t), mpmath.mpf(n)
        v = mpmath.exp(
            mpmath.loggamma((N + 1) / 2) - mpmath.loggamma(N / 2) - (N + 1) / 2 * mpmath.log1p(T * T / N)
        ) / mpmath.sqrt(N * mpmath.pi)
        return float(v)

"""verify-r3-loader-const: independent data loader for BETA.DIST pdf cells (a, b > 1).

Parses every case formula itself (no worker code): `=BETA.DIST(x, a, b, FALSE)` (optionally with A = 0, B = 1),
where each argument is an expression of number literals, same-sheet cell references, + - * / ^ and parentheses.
Formula number literals are truncated to 15 significant digits (Excel's formula parser); numeric cell values stored
as JSON numbers are exact doubles (the xlsx value path). Excel's value comes from the case's .oracle.json sidecar.
Rows are cached in vrows.pkl (git-ignored).
"""

from __future__ import annotations

import json
import pickle
import re
from decimal import Decimal
from pathlib import Path

HERE = Path(__file__).parent
REPO = HERE.parents[3]
SUITES = REPO / "eval" / "suites"
W1_PROBE = HERE.parent / "w1-density" / "probe"

_TOK = re.compile(
    r"\s*(?:(\d+\.?\d*(?:[eE][+-]?\d+)?|\.\d+(?:[eE][+-]?\d+)?)|([A-Z]{1,3}\$?\d+|\$?[A-Z]{1,3}\$?\d+)|(.))"
)
_CALL = re.compile(r"^=BETA\.DIST\((.*)\)$", re.I)


def trunc15(lit: str) -> float:
    """Excel's formula literal: keep the first 15 significant digits (truncate), then round to a double."""
    d = Decimal(lit)
    if d == 0:
        return 0.0
    sign, digits, exp = d.as_tuple()
    if len(digits) > 15:
        exp += len(digits) - 15
        digits = digits[:15]
    return float(Decimal((sign, digits, exp)))


class _P:
    """Recursive-descent evaluator (Excel precedence: unary minus > ^ > * / > + -; ^ left-associative)."""

    def __init__(self, text: str, ref):
        self.toks = []
        for m in _TOK.finditer(text):
            num, cell, op = m.groups()
            if num is not None:
                self.toks.append(("n", num))
            elif cell is not None:
                self.toks.append(("r", cell.replace("$", "")))
            elif op is not None and not op.isspace():
                self.toks.append(("o", op))
        self.i = 0
        self.ref = ref

    def peek(self):
        return self.toks[self.i] if self.i < len(self.toks) else ("e", "")

    def take(self):
        t = self.peek()
        self.i += 1
        return t

    def expr(self):
        v = self.term()
        while self.peek() in (("o", "+"), ("o", "-")):
            op = self.take()[1]
            w = self.term()
            v = v + w if op == "+" else v - w
        return v

    def term(self):
        v = self.power()
        while self.peek() in (("o", "*"), ("o", "/")):
            op = self.take()[1]
            w = self.power()
            v = v * w if op == "*" else v / w
        return v

    def power(self):
        v = self.unary()
        while self.peek() == ("o", "^"):
            self.take()
            v = v ** self.unary()
        return v

    def unary(self):
        if self.peek() == ("o", "-"):
            self.take()
            return -self.unary()
        if self.peek() == ("o", "+"):
            self.take()
            return self.unary()
        kind, val = self.take()
        if kind == "n":
            return trunc15(val)
        if kind == "r":
            return self.ref(val)
        if (kind, val) == ("o", "("):
            v = self.expr()
            if self.take() != ("o", ")"):
                raise ValueError("unbalanced")
            return v
        raise ValueError(f"unexpected {kind} {val}")


def split_args(s: str):
    out, depth, cur = [], 0, ""
    for ch in s:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return [a.strip() for a in out]


def eval_text(text: str, cells: dict) -> float:
    def ref(name, seen=()):
        v = cells.get(name)
        if isinstance(v, (int, float)) and not isinstance(v, bool):
            return float(v)
        if isinstance(v, str) and v.startswith("="):
            if name in seen:
                raise ValueError("cycle")
            return eval_text(v[1:], cells)
        raise ValueError(f"unresolvable ref {name}={v!r}")

    p = _P(text, ref)
    v = p.expr()
    if p.i != len(p.toks):
        raise ValueError(f"trailing tokens in {text}")
    return float(v)


def pdf_args(formula: str, cells: dict):
    """(x, a, b) of a BETA.DIST pdf formula on [0, 1], or None."""
    m = _CALL.match(formula.strip())
    if not m:
        return None
    args = split_args(m.group(1))
    if len(args) not in (4, 6) or args[3].upper() not in ("FALSE", "0"):
        return None
    try:
        x, a, b = (eval_text(t, cells) for t in args[:3])
        if len(args) == 6 and (eval_text(args[4], cells), eval_text(args[5], cells)) != (0.0, 1.0):
            return None
    except (ValueError, ZeroDivisionError, OverflowError):
        return None
    return x, a, b


def _case_files():
    yield from sorted(SUITES.glob("*/cases/*.case.json"))
    yield W1_PROBE / "probe1.case.json"


def _oracle_for(case: Path):
    o = case.with_name(case.name.replace(".case.json", ".oracle.json"))
    if o.exists():
        return json.load(open(o))["sheets"]
    o = case.with_name(case.name.replace(".case.json", ".out.json"))  # w1-density's probe output
    if o.exists():
        return json.load(open(o))["sheets"]
    return None


_CACHE = HERE / "vrows.pkl"


def load_all(refresh: bool = False):
    """Rows: dict(case, sheet, cell, x, a, b, excel) for every oracle-filled pdf cell with a > 1, b > 1, 0 < x < 1.
    Also returns parse stats per case (formulas seen / parsed)."""
    if _CACHE.exists() and not refresh:
        return pickle.loads(_CACHE.read_bytes())
    rows, stats = [], {}
    for case in _case_files():
        txt = case.read_text()
        if "BETA.DIST" not in txt.upper():
            continue
        c = json.loads(txt)
        orc = _oracle_for(case)
        if orc is None:
            continue
        name = case.name.replace(".case.json", "")
        seen = parsed = 0
        for sh in c.get("sheets", []):
            cells = sh.get("cells", {})
            out = orc.get(sh["name"], {})
            for cell, f in cells.items():
                if not (isinstance(f, str) and f.upper().startswith("=BETA.DIST(") and "FALSE" in f.upper()):
                    continue
                seen += 1
                xab = pdf_args(f, cells)
                if xab is None:
                    continue
                parsed += 1
                x, a, b = xab
                ov = out.get(cell)
                if not (a > 1 and b > 1 and 0 < x < 1) or not ov or ov.get("t") != "n":
                    continue
                rows.append(dict(case=name, sheet=sh["name"], cell=cell, x=x, a=a, b=b, excel=float(ov["v"])))
        stats[name] = (seen, parsed)
    _CACHE.write_bytes(pickle.dumps((rows, stats)))
    return rows, stats


def echo_check(case_name: str):
    """For the r3-loader-const probes: compare this parser's x, a, b with Excel's echo cells (B = x, C = a, D = b on
    the same row). Returns (checked, mismatches)."""
    case = SUITES / "probes-numeric" / "cases" / f"{case_name}.case.json"
    c = json.load(open(case))
    orc = _oracle_for(case)
    chk = bad = 0
    for sh in c["sheets"]:
        cells, out = sh["cells"], orc[sh["name"]]
        for cell, f in cells.items():
            if not cell.startswith("A"):
                continue
            xab = pdf_args(f, cells)
            if xab is None:
                continue
            row = cell[1:]
            for col, v in zip("BCD", xab):
                ov = out.get(col + row)
                if ov is not None and ov.get("t") == "n":
                    chk += 1
                    bad += float(ov["v"]) != v
    return chk, bad

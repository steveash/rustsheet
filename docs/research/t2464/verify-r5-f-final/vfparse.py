"""Verifier (T-2586, verify-r5-f-final): own parser for every F.DIST(x, d1, d2, FALSE|0) cell in probes-numeric.

A small Excel-formula evaluator (numbers with 15-significant-digit truncation, A1 refs, + - * / ^, unary minus binding
tighter than ^, EXP, TRUE/FALSE, "text" coerced to a number). Each op is one double rounding (Excel stores doubles).
A referenced x cell is evaluated from its own formula and checked against Excel's echo of it; the echo (the double
Excel actually passed) is used as x. A trailing '*k^j' scale on the F.DIST call is undone exactly.
rows(case) -> list of dict(case, cell, x, d1, d2, excel, echo_ok, xsrc)."""

from __future__ import annotations

import json
import math
import re
from fractions import Fraction
from pathlib import Path

CASES = Path(__file__).resolve().parents[4] / "eval" / "suites" / "probes-numeric" / "cases"
_TOK = re.compile(r'\s*(?:(\d+\.?\d*(?:[eE][-+]?\d+)?|\.\d+(?:[eE][-+]?\d+)?)|([A-Z]{1,3}\d+)|([A-Z][A-Z0-9.]*)\(|(TRUE|FALSE)\b|("[^"]*")|(.))')


def lit15(s: str) -> float:
    """Excel numeric literal: the decimal mantissa truncated (not rounded) to 15 significant digits, then rounded to
    the nearest double."""
    s = s.strip()
    mant, _, ex = s.upper().partition("E")
    e = int(ex) if ex else 0
    ip, _, fp = mant.partition(".")
    digits = (ip + fp).lstrip("0")
    if not digits:
        return 0.0
    allds = ip + fp
    lead = len(allds) - len(allds.lstrip("0"))
    keep = allds[: lead + 15]
    pad = len(allds) - len(keep)
    q = Fraction(int(keep) * 10**pad, 10 ** len(fp)) * Fraction(10) ** e
    return float(q)


class Unsupported(Exception):
    pass


class _P:
    def __init__(self, s: str, ref):
        self.t = []
        for m in _TOK.finditer(s):
            if m.group(1):
                self.t.append(("num", m.group(1)))
            elif m.group(4):
                self.t.append(("bool", m.group(4)))
            elif m.group(3):
                self.t.append(("fn", m.group(3)))
            elif m.group(2):
                self.t.append(("ref", m.group(2)))
            elif m.group(5):
                self.t.append(("str", m.group(5)[1:-1]))
            elif m.group(6) and m.group(6).strip():
                self.t.append(("op", m.group(6)))
        self.i, self.ref = 0, ref
        self.approx = False

    def peek(self):
        return self.t[self.i] if self.i < len(self.t) else (None, None)

    def take(self):
        tok = self.peek()
        self.i += 1
        return tok

    _OPS = {"+": lambda v, w: v + w, "-": lambda v, w: v - w, "*": lambda v, w: v * w, "/": lambda v, w: v / w}

    def _level(self, sub, ops):
        """One left-associative precedence level: sub (op sub)*, op in ops."""
        v = sub()
        while self.peek()[0] == "op" and self.peek()[1] in ops:
            v = self._OPS[self.take()[1]](v, sub())
        return v

    def expr(self):  # + -
        return self._level(self.term, "+-")

    def term(self):  # * /
        return self._level(self.power, "*/")

    def power(self):  # ^ (left-assoc in Excel)
        v = self.unary()
        while self.peek() == ("op", "^"):
            self.take()
            w = self.unary()
            if w != int(w):
                raise Unsupported("non-integer power")
            v = float(Fraction(v) ** int(w))  # exact power, one rounding (Excel's POWER is exact on these)
        return v

    def unary(self):
        if self.peek() == ("op", "-"):
            self.take()
            return -self.unary()
        if self.peek() == ("op", "+"):
            self.take()
            return self.unary()
        return self.atom()

    def atom(self):
        k, v = self.take()
        if k == "num":
            return lit15(v)
        if k == "bool":
            return 1.0 if v == "TRUE" else 0.0
        if k == "str":
            return lit15(v)
        if k == "ref":
            return self.ref(v)
        if k == "op" and v == "(":
            r = self.expr()
            if self.take() != ("op", ")"):
                raise Unsupported("paren")
            return r
        if k == "fn" and v == "EXP":
            a = self.expr()
            self.take()
            self.approx = True  # math.exp, not Excel's EXP (only used for subnormal rule cells)
            return math.exp(a)
        raise Unsupported(f"{k} {v}")


def evaluate(s: str, ref=lambda r: (_ for _ in ()).throw(Unsupported(r))):
    p = _P(s.lstrip("="), ref)
    v = p.expr()
    if p.i != len(p.t):
        raise Unsupported("trailing")
    return v, p.approx


def split_args(inner: str) -> list[str]:
    out, depth, cur, q = [], 0, "", False
    for ch in inner:
        if ch == '"':
            q = not q
        if not q and ch == "(":
            depth += 1
        if not q and ch == ")":
            depth -= 1
        if not q and ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    out.append(cur)
    return [a.strip() for a in out]


_CALL = re.compile(r"^=F\.DIST\((.*)\)(\*(\d+)\^(-?\d+))?$", re.I)


def _xl(o):
    if o is None:
        return None
    if o.get("t") == "n":
        return float(o["v"])
    if o.get("t") == "e":
        return o["v"]
    return None


def rows(case: str) -> list[dict]:
    doc = json.loads((CASES / f"{case}.case.json").read_text())
    orc_p = CASES / f"{case}.oracle.json"
    orc = json.loads(orc_p.read_text())["sheets"] if orc_p.exists() else {}
    out = []
    for sh in doc["sheets"]:
        cells, so = sh["cells"], orc.get(sh["name"], {})
        stats = {"echo_bad": 0}

        def ref(r, cells=cells, so=so, stats=stats):
            f = cells.get(r)
            mine = evaluate(f)[0] if isinstance(f, str) and f.startswith("=") else (float(f) if f is not None else 0.0)
            echo = _xl(so.get(r))
            if isinstance(echo, float):
                if echo != mine:
                    stats["echo_bad"] += 1
                    raise Unsupported(f"echo mismatch {r}")
                return echo
            return mine

        for cell, f in cells.items():
            if not isinstance(f, str):
                continue
            m = _CALL.match(f.replace(" ", ""))
            if not m or ".RT" in f.upper():
                continue
            args = split_args(m.group(1))
            if len(args) != 4 or args[3].upper() not in ("FALSE", "0"):
                continue
            try:
                (x, ax), (d1, _), (d2, _) = (evaluate(a, ref) for a in args[:3])
            except Unsupported:
                continue
            ex = _xl(so.get(cell)) if so else None
            if m.group(2) and isinstance(ex, str):
                continue  # an error after '*2^k' may be the scaling's own overflow, not F.DIST's
            if m.group(2) and isinstance(ex, float):
                q = Fraction(ex) / Fraction(int(m.group(3))) ** int(m.group(4))
                ex = float(q)
                if Fraction(ex) != q:
                    continue
            out.append(dict(case=case, cell=cell, x=x, d1=d1, d2=d2, excel=ex, xapprox=ax,
                            literal=not re.search(r"(?<![\d.])[A-Z]{1,3}\d+", args[0])))
        out_bad = stats["echo_bad"]
        if out_bad:
            print(f"# {case}: {out_bad} echo mismatches dropped")
    return out


def all_cases() -> list[str]:
    ids = []
    for p in sorted(CASES.glob("*.case.json")):
        t = p.read_text()
        if "F.DIST(" in t.upper():
            ids.append(p.name[: -len(".case.json")])
    return ids

"""verify-r5-f-combine: an independent parser of probes-numeric F.DIST(x, d1, d2, FALSE) cells (no worker code).

A row is one cell whose formula is exactly =F.DIST(<arg>, d1, d2, FALSE|0) with numeric d1, d2 and a positive numeric
oracle value. <arg> is a cell reference (x = the oracle's echo of that cell, which must itself be an exact two-piece
formula or a literal we re-derive) or a numeric literal (Excel truncates formula literals to 15 significant digits,
then rounds to the nearest double). d1, d2 are truncated to integers as F.DIST does.
"""

from __future__ import annotations

import json
import math
import re
from decimal import ROUND_DOWN, Decimal
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CASES = ROOT / "eval" / "suites" / "probes-numeric" / "cases"
_NUM = r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"
_FD = re.compile(rf"^=F\.DIST\(\s*([A-Z]+\d+|{_NUM})\s*,\s*({_NUM})\s*,\s*({_NUM})\s*,\s*(FALSE|0)\s*\)$", re.I)
_TWO = re.compile(r"^=\((\d+)\*2\^-(\d+)\+(\d+)\*2\^-(\d+)\)$")
_ONE = re.compile(r"^=\(?(\d+)\*2\^-(\d+)\)?$")


def lit15(s: str) -> float:
    """Excel's reading of a formula literal: truncate to 15 significant digits, then nearest double."""
    d = Decimal(s)
    if d == 0:
        return 0.0
    q = Decimal(1).scaleb(d.adjusted() - 14)
    return float(d.quantize(q, rounding=ROUND_DOWN))


def two_piece(f: str) -> float | None:
    f = f.replace(" ", "")
    m = _TWO.match(f) or _ONE.match(f)
    if not m:
        return None
    a, ea, b, eb = (tuple(int(g) for g in m.groups()) + (0, 0))[:4]
    v = Fraction(a, 2**ea) + Fraction(b, 2**eb)
    x = float(v)
    return x if Fraction(x) == v else None  # exact by construction, else refuse


def oracle_vals(case_id: str) -> dict:
    p = CASES / f"{case_id}.oracle.json"
    if not p.exists():
        return {}
    sh = json.loads(p.read_text())["sheets"]
    out = {}
    for s, cells in sh.items():
        for c, v in cells.items():
            if isinstance(v, dict) and v.get("t") == "n":
                out[(s, c)] = float(v["v"])
    return out


def rows(case_id: str, need_exact_x: bool = True) -> list[dict]:
    doc = json.loads((CASES / f"{case_id}.case.json").read_text())
    ov = oracle_vals(case_id)
    out = []
    for sheet in doc["sheets"]:
        cells = sheet["cells"]
        for cell, f in cells.items():
            if not isinstance(f, str):
                continue
            m = _FD.match(f.strip())
            if not m:
                continue
            arg, s1, s2, _ = m.groups()
            exv = ov.get((sheet["name"], cell))
            if exv is None or not exv > 0 or not math.isfinite(exv):
                continue
            if re.match(r"^[A-Z]+\d+$", arg, re.I):
                src = cells.get(arg.upper())
                if isinstance(src, (int, float)):
                    x = float(src)
                elif isinstance(src, str):
                    x = two_piece(src) if src.startswith("=") else lit15(src)
                else:
                    x = None
                echo = ov.get((sheet["name"], arg.upper()))
                if x is None:
                    if need_exact_x:
                        continue
                    x = echo
                if echo is None or echo != x:
                    continue  # unverified or misread input
            else:
                x = lit15(arg)
            d1, d2 = math.trunc(lit15(s1)), math.trunc(lit15(s2))
            if x is None or not x > 0 or d1 < 1 or d2 < 1:
                continue
            out.append(dict(case=case_id, cell=cell, x=x, d1=float(d1), d2=float(d2), excel=exv))
    return out


def fdist_cases() -> list[str]:
    return sorted(p.name[: -len(".case.json")] for p in CASES.glob("*.case.json") if "F.DIST(" in p.read_text())

"""verify-r5-f-smallr: independent parser for F.DIST(x, d1, d2, FALSE) cells in probes-numeric cases.

Own code (no worker code). x comes from the referenced cell's oracle echo; the echo is cross-checked against an exact
evaluation of the cell's formula when it is the two-piece encoder '=(p*2^-k+q*2^-j)' (or a plain literal).
Numeric literals are read the way Excel reads them: truncated (not rounded) to 15 significant digits."""
from __future__ import annotations

import json
import re
from decimal import ROUND_DOWN, Decimal
from fractions import Fraction
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CASES = ROOT / "eval" / "suites" / "probes-numeric" / "cases"
NUM = r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"
FDIST = re.compile(rf"^=F\.DIST\(\s*([A-Z]+\d+|{NUM})\s*,\s*({NUM})\s*,\s*({NUM})\s*,\s*(FALSE|0)\s*\)$", re.I)
ENC1 = re.compile(r"^=\(?(\d+)\*2\^(-?\d+)\)?$")
ENC = re.compile(r"^=\(?\s*(\d+)\s*\*\s*2\^(-?\d+)\s*\+\s*(\d+)\s*\*\s*2\^(-?\d+)\s*\)?$")


def lit15(s: str) -> float:
    """Excel's reading of a numeric literal: truncate to 15 significant digits, then round to double."""
    d = Decimal(s)
    if d != 0 and len(d.as_tuple().digits) > 15:
        d = d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_DOWN)
    return float(Fraction(d)) if d != 0 else 0.0


def eval_formula(f: str):
    """Exact double of an x-encoder formula, or None if not recognised."""
    if not f.startswith("="):
        try:
            return lit15(f)
        except Exception:
            return None
    m = ENC.match(f.replace(" ", ""))
    if m:
        p, k, q, j = (int(g) for g in m.groups())
        a = Fraction(p) * Fraction(2) ** k
        b = Fraction(q) * Fraction(2) ** j
        if float(a) != a or float(b) != b:  # each piece must be exact in double
            return None
        return float(a + b)  # one rounding of the exact sum
    m = ENC1.match(f.replace(" ", ""))
    if m:
        p, k = int(m.group(1)), int(m.group(2))
        if len(m.group(1)) > 15:  # Excel truncates longer literals; not used by the encoders
            return None
        return float(Fraction(p) * Fraction(2) ** k)
    if re.fullmatch(NUM, f[1:]):
        return lit15(f[1:])
    return None


def rows(case_id: str):
    """[(cell, x, d1, d2, excel)] and a stats dict for one case (id without .case.json)."""
    c = json.loads((CASES / f"{case_id}.case.json").read_text())
    op = CASES / f"{case_id}.oracle.json"
    if not op.exists():
        return [], {"no_oracle": 1}
    o = json.loads(op.read_text())
    out, st = [], {"cells": 0, "echo_ok": 0, "echo_bad": 0, "echo_unk": 0, "nonnum": 0}
    for sh in c["sheets"]:
        cells = sh["cells"]
        ov = o["sheets"].get(sh["name"], {})
        for cell, f in cells.items():
            if not isinstance(f, str):
                continue
            m = FDIST.match(f.strip())
            if not m:
                continue
            st["cells"] += 1
            ref, d1, d2, _ = m.groups()
            e = ov.get(cell)
            if not e or e.get("t") != "n":
                st["nonnum"] += 1
                continue
            if re.fullmatch(r"[A-Z]+\d+", ref, re.I):
                echo = ov.get(ref.upper())
                if not echo or echo.get("t") != "n":
                    st["nonnum"] += 1
                    continue
                x = float(echo["v"])
                src = cells.get(ref.upper())
                ex = eval_formula(src) if isinstance(src, str) else (float(src) if isinstance(src, (int, float)) else None)
                if ex is None:
                    st["echo_unk"] += 1
                elif ex == x:
                    st["echo_ok"] += 1
                else:
                    st["echo_bad"] += 1
                    continue  # misread input: drop (README: xlsx parser misreads ~1e-6)
            else:
                x = lit15(ref)
            out.append((cell, x, lit15(d1), lit15(d2), float(e["v"])))
    return out, st


def all_case_ids():
    ids = []
    for p in sorted(CASES.glob("*.case.json")):
        if "F.DIST(" in p.read_text():
            ids.append(p.name[: -len(".case.json")])
    return ids

"""T-2512 r3-bgrat: BETA.DIST cdf cells read straight from probes-numeric case + oracle files (no derived tables).

Formula literals are decoded as Excel reads them: w2-forensics' exact forms M*2^E and (H*2^e1+L*2^e2) exactly, decimal
literals truncated (not rounded) to 15 significant digits. Only BETA.DIST(x, a, b, TRUE) with the default bounds.
Rows: dict(case, cell, x, a, b, y = fl(1 - x), excel, hold = md5(case, cell) odd).
"""
from __future__ import annotations

import hashlib
import json
import math
import re
from decimal import Decimal
from pathlib import Path

CASES = Path(__file__).resolve().parents[4] / "eval" / "suites" / "probes-numeric" / "cases"
_ARG = re.compile(r"^=BETA\.DIST\((.*),(TRUE|FALSE)\)$")


def lit(s: str) -> float:
    s = s.strip()
    m = re.fullmatch(r"\((\d+)\*2\^(-?\d+)\+(\d+)\*2\^(-?\d+)\)", s)
    if m:
        return math.ldexp(int(m[1]), int(m[2])) + math.ldexp(int(m[3]), int(m[4]))
    m = re.fullmatch(r"(\d+)\*2\^(-?\d+)", s)
    if m:
        return math.ldexp(int(m[1]), int(m[2]))
    d = Decimal(s)
    if len(d.as_tuple().digits) > 15:  # Excel's parser truncates to 15 significant digits
        sign, digits, exp = d.as_tuple()
        d = Decimal((sign, digits[:15], exp + len(digits) - 15))
    return float(d)


def hold(case: str, cell: str) -> bool:
    return int(hashlib.md5(repr((case, cell)).encode()).hexdigest(), 16) % 2 == 1


def load(pattern: str = "*.case.json"):
    rows = []
    for p in sorted(CASES.glob(pattern)):
        o = p.with_name(p.name.replace(".case.json", ".oracle.json"))
        if not o.exists():
            continue
        case = json.loads(p.read_text())
        out = json.loads(o.read_text())["sheets"]
        for sh in case["sheets"]:
            got = out.get(sh["name"], {})
            for cell, f in sh["cells"].items():
                if not isinstance(f, str):
                    continue
                m = _ARG.match(f.replace(" ", ""))
                if not m or m[2] != "TRUE":
                    continue
                parts = m[1].split(",")
                if len(parts) != 3:
                    continue
                try:
                    x, a, b = (lit(t) for t in parts)
                except Exception:  # noqa: BLE001  (nested formulas: skip)
                    continue
                c = got.get(cell)
                if not c or c.get("t") != "n" or not (0 < x < 1) or a <= 0 or b <= 0:
                    continue
                rows.append(dict(case=p.name.replace(".case.json", ""), cell=cell, x=x, a=a, b=b, y=1.0 - x,
                                 excel=float(c["v"]), hold=hold(p.name, cell)))
    return rows


if __name__ == "__main__":
    rs = load()
    from collections import Counter
    print(len(rs)); print(Counter(r["case"] for r in rs).most_common(60))

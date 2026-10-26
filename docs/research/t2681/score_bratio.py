"""T-2681: score T-2464 r2-bratio's integrated model (`bratio_excel.BEST`) as F.TEST's tail.

Usage: T2464_HARNESS=<rustsheet_engine test binary> python score_bratio.py CELLS.json
(CELLS.json from dump_cells.py; the python needs mpmath and numpy on x86-64, see
docs/research/t2464/r2-bratio/bratio_excel.py). Prints, per bratio branch, the bratio and engine
counts (bit-exact, within 4 ulp, visibly off), then the hybrid fisher.rs ships: bratio for both
shapes above 1 where `f_test_bratio_p` reaches, the engine's value elsewhere.
"""

import collections
import json
import math
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "t2464" / "r2-bratio"))
import bratio_excel as B  # noqa: E402


def ordered(v: float) -> int:
    i = struct.unpack("<q", struct.pack("<d", v))[0]
    return i if i >= 0 else -(i & 0x7FFFFFFFFFFFFFFF)


def ulp(p: float, q: float) -> int:
    return abs(ordered(p) - ordered(q))


def visibly_off(p: float, e: float) -> bool:
    """Decision 0083: more than half a unit in the 15th significant digit of Excel's value."""
    return p != 0 if e == 0 else abs(p - e) > 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14)


def counts(rows: list[tuple[float, float]]) -> str:
    exact = sum(ulp(p, e) == 0 for p, e in rows)
    close = sum(ulp(p, e) <= 4 for p, e in rows)
    return f"{exact} / {close} / {sum(visibly_off(p, e) for p, e in rows)} of {len(rows)}"


def ported(a: float, b: float, x: float, y: float) -> bool:
    """Where fisher.rs `f_test_bratio_p` returns a value (its `None` cases keep the engine)."""
    if min(a, b) <= 1.0 or a + b > 200.0:
        return False
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    a0, b0, x0 = (b, a, y) if lam < 0 else (a, b, x)
    if b0 >= 40.0:
        return False
    if b0 * x0 <= 0.7:
        return True
    shape = b0 - int(b0) or 1.0
    return x0 <= 0.7 and not (shape == 1.0 and a0 <= 2.0)  # gsumln is not ported


cells = json.load(open(sys.argv[1]))
results = B.run([(c["a"], c["b"], c["x"], c["y"]) for c in cells], B.BEST)
by_branch = collections.defaultdict(list)
hybrid = []
for c, (w, w1, branch) in zip(cells, results):
    p = 2.0 * min(w, w1)
    by_branch[branch].append((c, p))
    hybrid.append((p if ported(c["a"], c["b"], c["x"], c["y"]) else c["engine"], c["excel"]))
print("branch                bratio            engine   (bit-exact / <=4 ulp / visibly off)")
for branch, rows in sorted(by_branch.items()):
    print(f"{branch:20} {counts([(p, c['excel']) for c, p in rows]):>18} {counts([(c['engine'], c['excel']) for c, _ in rows]):>18}")
print("engine:", counts([(c["engine"], c["excel"]) for c in cells]))
print("hybrid:", counts(hybrid))

"""T-3572 r5-gamma-p1: the engine's misses on probe t2512-r3-bgrat-qgamma column A, GAMMA.DIST(z, b, 1, TRUE) at b < 1.

usage: rustsheet-eval show --all probes-numeric/t2512-r3-bgrat-qgamma.case.json > show.txt; ulps.py show.txt [cells.tsv]
Prints the non-passing cells by ulp distance (spacing of doubles at Excel's value) and by whether they are visibly off
(outside half a unit of the 15th significant digit of Excel's value), and writes the failing cells' b, z and values."""
import math
import re
import struct
import sys
from collections import Counter


def ordinal(x: float) -> int:
    n = struct.unpack("<q", struct.pack("<d", x))[0]
    return n if n >= 0 else -(n & 0x7FFFFFFFFFFFFFFF)


def w15(a: float, e: float) -> bool:
    """True when a and e agree to 15 significant digits (a rounds to e's 15-digit truncation)."""
    return float(f"{a:.14e}") == float(f"{e:.14e}")


cells = []
cur = None
for line in open(sys.argv[1]):
    m = re.match(r"Calc!(A\d+)\s+(Pass|NumericUlp)", line)
    if m:
        cur = {"cell": m.group(1), "st": m.group(2)}
        cells.append(cur)
    elif cur:
        for k in ("formula", "expected", "actual"):
            if re.match(k + r"\s*:", line.strip()):
                cur[k] = line.split(":", 1)[1].strip()
tot = Counter()
ulp = Counter()
vis = 0
rows = []
for c in cells:
    if "formula" not in c or "GAMMA.DIST" not in c["formula"]:
        continue
    tot[c["st"]] += 1
    if c["st"] == "Pass":
        continue
    e, a = float(c["expected"]), float(c["actual"])
    d = abs(ordinal(a) - ordinal(e))
    ulp[min(d, 5)] += 1
    v = not w15(a, e)
    vis += v
    rows.append((c["cell"], d, v, c["expected"], c["actual"], c["formula"]))
print("column A:", dict(tot))
print("ulp distance (5 = 5 or more):", dict(sorted(ulp.items())))
print("off at the 15th digit:", vis)
if len(sys.argv) > 2:
    with open(sys.argv[2], "w") as f:
        for r in rows:
            f.write("\t".join(map(str, r)) + "\n")

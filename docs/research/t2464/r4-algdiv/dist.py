"""T-3383 r4-algdiv: distribution of (pinned lnB - correctly rounded lnB) in ulps of lnB per shape, and the
neighbour correlation of that offset along the fine b steps (smooth curve = approximation error; none = storage).
Reads scan_<name>.json (scan.py). Usage: dist.py a hold"""

import collections
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent

for name in sys.argv[1:]:
    rows = json.loads((HERE / f"scan_{name}.json").read_text())
    ok = [r for r in rows if r[5] == r[2]]  # shapes whose pin is at 100%
    d = collections.Counter(r[4] - r[7] for r in ok)
    print(name, "pinned - CR lnB (ulps of lnB) over", len(ok), "shapes:", dict(sorted(d.items())))
    for a in sorted({r[0] for r in ok}):
        fine = [r[4] - r[7] for r in ok if r[0] == a and r[1] == int(r[1] * 8) / 8 and r[1] >= 16 and r[1] < 44.5]
        if len(fine) > 2:
            m = sum(fine) / len(fine)
            num = sum((x - m) * (y - m) for x, y in zip(fine, fine[1:]))
            den = sum((x - m) ** 2 for x in fine) or 1
            print(f"  a={a}: fine-step offsets {fine}  lag-1 autocorrelation {num / den:+.2f}")

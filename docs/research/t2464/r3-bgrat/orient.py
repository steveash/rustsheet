"""T-2512 r3-bgrat: Excel's beta density in the two orientations at the -bupdens points: BETA.DIST(x, a, b0, FALSE)
against BETA.DIST(y, b0, a, FALSE), in ulps of the second (the bup prefactor's own orientation)."""
from __future__ import annotations

import math
from collections import Counter

from common import bupdens_p

d = [(pa - pb) / math.ulp(pb) for pa, pb in bupdens_p().values() if pa is not None and pb is not None]
print(f"points {len(d)}, differ {sum(v != 0 for v in d)}, min {min(d):+.0f}, max {max(d):+.0f} ulp")
print("ulp: count", sorted(Counter(round(v) for v in d).items()))

"""Do GL53's remaining misses on t3570-r5-bpser-lngamma-beta follow a0 (so L) or x (so the series or EXP)?

For each (b0, a0) the probe has two x. If L were wrong at an a0, both x would tend to miss together. Prints, per
(b0, center), the misses at each x, the a0 where both miss, the count expected if misses were independent, and the
signed ulp offsets of the misses (lshift.py tries constant L shifts).
usage: residual.py
"""

from __future__ import annotations

import json
import math
from collections import Counter, defaultdict

import make_probe as mp
import score_forms as score

lib = mp.lib


def main() -> None:
    lib.bx._GL.update(score.measured_gl())
    out = score.sidecar("t3570-r5-bpser-lngamma-beta")
    v = lib.V(lg="GL53")
    miss = defaultdict(dict)
    offs = defaultdict(Counter)
    for i, blk, x, a, b in json.load(open(mp.HERE / "t3570-r5-bpser-lngamma-beta.meta.json")):
        e = float(out[f"A{i}"]["v"])
        m = lib.kernel(a, b, x, 1.0 - x, v)[0]
        b0, c, xs = blk.split(":")
        miss[(b0, c)][(a, xs)] = m != e
        if m != e:
            offs[(b0, c)][round((m - e) / math.ulp(e))] += 1
    print(f"{'b0:center':10} {'miss x1':>7} {'miss x2':>7} {'both':>5} {'indep':>6}  offsets (model - excel, ulps)")
    for key in sorted(miss):
        d = miss[key]
        xs = sorted({k[1] for k in d})
        a0s = sorted({k[0] for k in d})
        m1 = sum(d[(a, xs[0])] for a in a0s)
        m2 = sum(d[(a, xs[1])] for a in a0s)
        both = sum(d[(a, xs[0])] and d[(a, xs[1])] for a in a0s)
        print(f"{':'.join(key):10} {m1:>7} {m2:>7} {both:>5} {m1 * m2 / len(a0s):>6.1f}  {dict(sorted(offs[key].items()))}")


if __name__ == "__main__":
    main()

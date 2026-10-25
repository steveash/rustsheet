"""Misses of the frozen M-sd ('div') and M-sdm ('mulr') by ulp distance, and whether the model at x +- 1 ulp
matches Excel (the xlsx parser quirk). Usage: s22_ulp.py SET [SET ...]"""
import collections
import math
import sys

import sdmodel as SD
import slib as L

for s in sys.argv[1:]:
    rows = L.load(s)
    for fin in ("div", "mulr"):
        c = collections.Counter()
        nb = 0
        for r in rows:
            v = SD.value(r["x"], r["d1"], r["d2"], fin)
            if v == r["excel"]:
                continue
            k = round((r["excel"] - v) / math.ulp(r["excel"]))
            c["1" if abs(k) == 1 else ("2-4" if abs(k) <= 4 else ">4")] += 1
            for dx in (-1, 1):
                if SD.value(math.nextafter(r["x"], dx * math.inf), r["d1"], r["d2"], fin) == r["excel"]:
                    nb += 1
                    break
        print(s, fin, "misses by ulp:", dict(c), "| matched at x +- 1 ulp:", nb)

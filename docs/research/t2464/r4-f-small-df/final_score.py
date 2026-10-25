"""Scores (bit-exact / within 4 ulp / within 15 digits, by class) of CR, r4-f-brcomp's M-xd and the frozen M-sd.
Usage: final_score.py SET [SET ...]   (sets: slib.load names; 'myhold' is the hold-out, read once)"""
import sys

import sdmodel as SD
import slib as L


def cls(r):
    d1, d2 = r["d1"], r["d2"]
    if max(d1, d2) >= 50:
        return "max>=50"
    if min(d1, d2) <= 2:
        return "min<=2"
    if max(d1, d2) <= 10:
        return "both<=10"
    return "other"


MODELS = {"CR": lambda r: r["cr"], "M-xd": L.mxd, "M-sd": lambda r: SD.value(r["x"], r["d1"], r["d2"]),
          "M-sdm": lambda r: SD.value(r["x"], r["d1"], r["d2"], "mulr")}
for s in sys.argv[1:]:
    rows = L.load(s)
    print(f"# {s}: {len(rows)} rows")
    for k, fn in MODELS.items():
        sc = L.score_rows(rows, fn, cls)
        print(f"{s:7s} {k:6s} all {L.F.fmt(sc['all'])} | " + " | ".join(f"{c} {L.F.fmt(sc[c])}" for c in ("min<=2", "both<=10", "other", "max>=50") if c in sc))

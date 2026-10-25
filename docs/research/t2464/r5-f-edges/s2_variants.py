"""Frozen round-4/5 models by edge region: M-sd, M-sdm (r4-f-small-df), hkp-sd, hk-sd (r5-f-combine, imported).
Usage: s2_variants.py SET [SET ...]"""
import sys

import elib as E

sys.path.insert(0, str(E.T / "r5-f-combine"))
CF = E._load("r5c_final", E.T / "r5-f-combine" / "final_score.py")
M = {"CR": lambda r: r["cr"], "M-sd": lambda r: E.SD.value(r["x"], r["d1"], r["d2"]),
     "M-sdm": lambda r: E.SD.value(r["x"], r["d1"], r["d2"], "mulr"),
     "hkp-sd": lambda r: CF.value(r, "hkp-sd"), "hk-sd": lambda r: CF.value(r, "hk-sd")}
for s in sys.argv[1:]:
    E.report(s, E.L.load(s), M)

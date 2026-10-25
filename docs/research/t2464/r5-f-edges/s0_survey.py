"""Survey: frozen M-sd (r4-f-small-df) and CR on existing sets by edge region. Usage: s0_survey.py SET [SET ...]"""
import sys

import elib as E

M = {"CR": lambda r: r["cr"], "M-sd": lambda r: E.SD.value(r["x"], r["d1"], r["d2"])}
for s in sys.argv[1:]:
    rows = E.L.load(s)
    E.report(s, rows, M)

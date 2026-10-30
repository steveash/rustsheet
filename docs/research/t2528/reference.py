"""T-2528 references: 120-digit mpmath I at the double arguments the engine and Excel see.

The F upper tail at df1 = 5 is betainc(5/2, df2/2, u, 1) with u = fl(5x / (5x + df2)); at 60 digits
mpmath's betainc is itself about 4E-14 off at x = 50, so 120 are used. Prints df2, x, the reference
and Excel's value (probes-numeric/t2528-f-dist-huge-df2) with its relative error.

Run: python3 -I docs/research/t2528/reference.py  (needs mpmath)
"""

import json
from pathlib import Path

import mpmath as mp

mp.mp.dps = 120
ORACLE = Path(__file__).resolve().parents[3] / (
    "eval/suites/probes-numeric/cases/t2528-f-dist-huge-df2.oracle.json"
)
excel = json.loads(ORACLE.read_text())["sheets"]["Calc"]
row = 1
for df2 in (1e10, 1e9, 1e8):
    for x in (2.0, 5.0, 10.0, 20.0, 50.0):
        scaled = 5 * x
        u = scaled / (scaled + df2)
        ref = mp.betainc(mp.mpf(2.5), mp.mpf(df2) / 2, mp.mpf(u), 1, regularized=True)
        upper = excel[f"A{row + 1}"]["v"]
        print(df2, x, mp.nstr(ref, 20), upper, float(upper / ref - 1))
        row += 4

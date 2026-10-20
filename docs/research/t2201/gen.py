"""T-2201 probe generator: MROUND near-half quotients. seed 2201."""
import json, math, random
from fractions import Fraction
from decimal import Decimal
random.seed(2201)
rows = []  # (number, multiple, companion_decimal or None, group)
def add(n, m, comp=None, g=""):
    rows.append((float(n), float(m), comp, g))
# group 1: T-2167 near-half quotients and negatives
for n, m in [(0.3,0.2),(0.7,0.2),(2.5,1),(1.15,0.1),(1.05,0.1)]:
    add(n, m, g="t2167"); add(-n, -m, g="t2167-neg")
# group 2: exact quotients (multiple 1): anchors +- j ulp, separates 15-digit cut from ulp tolerance
for anchor in [1.5, 2.5, 1234.5, 98765432.5]:
    for j in [1,2,4,8,12,16,20,24,32,48]:
        add(anchor - j*math.ulp(anchor), 1, g="exact-below")
        add(anchor + j*math.ulp(anchor), 1, g="exact-above")
# group 3: seeded pairs whose binary quotient lies within 4 ulp of k+0.5
mults = [0.1,0.2,0.3,0.7,0.05,0.15,0.35,1.1,1.3,2.7,0.07,0.03,0.9,1.7,0.45]
seen = 0; tries = 0
while seen < 80 and tries < 100000:
    tries += 1
    m = random.choice(mults); k = random.randint(1, 2000)
    n0 = float(Decimal(repr(m)) * (k + Decimal("0.5")))
    n = n0 + random.randint(-3, 3) * math.ulp(n0)
    q = n / m; h = k + 0.5
    d = (q - h) / math.ulp(h)
    if abs(d) <= 4 and d != 0:
        add(n, m, g="seeded"); seen += 1
# group 4: product forming, with decimal companions
for n, m, dec in [(0.5,0.2,"0.6"),(0.7,0.1,"0.7"),(1e15+0.3,0.2,"1000000000000000.2"),(0.35,0.1,"0.4"),
                  (1.1,0.3,"1.2"),(10,0.3,"9.9"),(0.29,0.1,"0.3"),(7.1,0.7,"7")]:
    add(n, m, comp=dec, g="product")
# group 5: controls
for n, m in [(3.5e-15,1e-15),(2.5e15,1e15),(1.23456e-14,1e-15),(7.5e15,1e15),(-3.5e-15,-1e-15)]:
    add(n, m, g="control")
cells = {}
for i, (n, m, comp, g) in enumerate(rows, start=1):
    cells[f"A{i}"] = n; cells[f"B{i}"] = m; cells[f"C{i}"] = f"=MROUND(A{i},B{i})"
    if comp is not None:
        cells[f"D{i}"] = f"=(C{i})={comp}"
case = {"format": "rustsheet-case/v1",
 "description": "T-2201 probe: MROUND quotient rounding near half-multiples and product forming. Stored inputs A (number), B (multiple); C = MROUND(A,B); D = (C)=<decimal> companions on the product rows. Rows 1-10: T-2167 near-half quotients and negatives; 11-90: exact quotients (multiple 1) at k+0.5 +- {1,2,4,8,12,16,20,24,32,48} ulp for k+0.5 in 1.5, 2.5, 1234.5, 98765432.5; 91-170: 80 seeded pairs (Python random.seed(2201), multiples from a fixed list, n = nearest double of m*(k+0.5) moved by -3..3 ulp, kept when n/m lies within 4 ulp of k+0.5, generator at docs/research/t2201/gen.py); 171-178: product forming; 179-183: controls with multiples 1E-15 and 1E+15.",
 "tags": ["M5", "probe", "math"], "sheets": [{"name": "M", "cells": cells}]}
print(len(rows), sum(1 for r in rows if r[2]) )
json.dump(case, open("eval/suites/probes/cases/t2201-mround-quotient-rounding.case.json", "w"), indent=1)
# group 6 (second fill): pin the tolerance t in frac(q) >= 0.5 - t between 4.44E-15 and 5.33E-15
extra = []
u = math.ulp(0.4)  # 2^-54, the ulp just below 0.5
for j in range(80, 97):
    extra.append((0.5 - j*u, 1.0))
for j in (21, 22, 23):
    extra.append((1.5 - j*math.ulp(1.0), 1.0))
for j in (88, 89, 90, 91, 92):
    extra.append((2*(0.5 - j*u), 2.0))
    extra.append((-(0.5 - j*u), -1.0))
start = len(rows) + 1
for k, (n, m) in enumerate(extra):
    i = start + k
    cells[f"A{i}"] = n; cells[f"B{i}"] = m; cells[f"C{i}"] = f"=MROUND(A{i},B{i})"
case["description"] += f" Rows {start}-{start+len(extra)-1} (second fill): fractional parts 0.5 - j*2^-54 (j = 80..96) and 1.5 - j*2^-52 (j = 21..23), plus multiple 2 and negatives at j = 88..92, which pin the tolerance."
json.dump(case, open("eval/suites/probes/cases/t2201-mround-quotient-rounding.case.json", "w"), indent=1)
print(start, len(extra))

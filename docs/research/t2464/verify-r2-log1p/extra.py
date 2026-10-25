"""Discordance (model vs Horner) per case, and dedupe of the cells.tsv out-of-sample rows."""
import csv, math, sys
from collections import Counter
import vscore as V
import excelmath
rows = [r for nm in ("runs", "grid", "hold") for r in V.load_case(nm) if not isinstance(r["B"], str) and r["p"] <= 0.375]
for alt in (dict(horner=True), dict(order="rev"), dict(quot="ieee")):
    a = [V.model(r["p"]) for r in rows]; b = [V.model(r["p"], **alt) for r in rows]
    ea = excelmath.excel("EXP", [r["n"] * l for r, l in zip(rows, a)])
    eb = excelmath.excel("EXP", [r["n"] * l for r, l in zip(rows, b)])
    for c in ("runs", "grid", "hold"):
        idx = [j for j, r in enumerate(rows) if r["case"] == c]
        mw = sum(ea[j] == rows[j]["B"] and eb[j] != rows[j]["B"] for j in idx)
        aw = sum(eb[j] == rows[j]["B"] and ea[j] != rows[j]["B"] for j in idx)
        print(alt, c, "model-only right", mw, "alt-only right", aw)
oos = []
for r in csv.DictReader(open(V.HERE.parent / "cells.tsv"), delimiter="\t"):
    if r["fn"] not in ("BINOM.DIST", "BINOMDIST"): continue
    a = r["args"].split("|")
    try: k, n, p = float(a[0]), float(a[1]), float(a[2])
    except Exception: continue
    if math.trunc(k) == 0 and 0 < p < 1: oos.append((r["fn"], r["case"], r["cell"], r["args"], p))
print("distinct p:", sorted(Counter(x[4] for x in oos).items()))
print("distinct (case,cell):", len({(x[1], x[2]) for x in oos}), "fn split", Counter(x[0] for x in oos))
print("sample:", oos[:4])

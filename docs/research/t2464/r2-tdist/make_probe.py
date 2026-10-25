"""Oracle probe t2464-r2-tdist-p1 (probes-numeric). Sheet Calc, column A formulas; meta rows (row, part, args):
  A  BETA.DIST(y, n/2, 0.5, FALSE) at every bup+bgrat T cell's y (Excel's density, the bup prefactor)
  B  T.DIST.RT(t, n) with z = t^2/(n + t^2) straddling 0.01 (t = sqrt(n/99) (1 + j 2e-7)), n in 40..3000
  C  T.DIST.RT out of sample in the bgrat branch: integer n in [31, 2000], t in [1, 8] (seeded)
  D  T.DIST.RT out of sample, n in [3, 30], t in [0.05, 12] (bpser / bup+bgrat / w1_bpser branches)
Exact doubles are written with w2-forensics' enc() (exact arithmetic of exact pieces).
usage: make_probe.py [--write]"""
import importlib.util
import json
import math
import random
import sys

import tscore
import tlib

spec = importlib.util.spec_from_file_location("w2probes", tlib.T2464 / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(w2p)
NAME = "t2464-r2-tdist-p1"

rows_meta = []
cells = {}


def add(part, formula, args):
    i = len(cells) + 1
    cells[f"A{i}"] = formula
    rows_meta.append((i, part, args))


rows = tscore.rows_L()
_, brs = tscore.evaluate(rows, "args=t2,alnrel=xl,erfc=xl,bg_lnx=cut0.01")
seen = set()
for r, b in zip(rows, brs):
    if b != "bup+bgrat/sw":
        continue
    tt, u, y, z = tlib.targs(abs(r["t"]), r["n"])
    if (y, r["n"]) in seen:
        continue
    seen.add((y, r["n"]))
    add("A", f"=BETA.DIST({w2p.num(y)},{w2p.num(r['n'] / 2)},0.5,FALSE)", (y, r["n"]))
for n in (40.0, 64.0, 100.0, 200.0, 300.0, 1000.0, 3000.0):
    t0 = math.sqrt(n / 99)
    for j in range(-200, 200):
        t = t0 * (1 + j * 2e-7)
        add("B", f"=T.DIST.RT({w2p.num(t)},{n:g})", (t, n))
rng = random.Random(2464)
for _ in range(5000):
    n = float(rng.randint(31, 2000))
    t = 1 + 7 * rng.random()
    add("C", f"=T.DIST.RT({w2p.num(t)},{n:g})", (t, n))
for _ in range(4000):
    n = float(rng.randint(3, 30))
    t = math.exp(rng.uniform(math.log(0.05), math.log(12)))
    add("D", f"=T.DIST.RT({w2p.num(t)},{n:g})", (t, n))
case = {"format": "rustsheet-case/v1",
        "description": "T-2464 r2-tdist: Student t kernel probes (bup density, bgrat z = 0.01 cut, out-of-sample tails)",
        "tags": ["M9", "probe", "statistics", "T-2464", "incomplete-beta"],
        "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
assert len(cells) <= 16384
p = w2p.CASES / f"{NAME}.case.json"
print(len(cells), "cells", p, "exists" if p.exists() else "")
if "--write" in sys.argv:
    assert not p.exists()
    p.write_text(json.dumps(case, indent=1) + "\n")
    json.dump(rows_meta, open(tlib.HERE / f"{NAME}.meta.json", "w"))

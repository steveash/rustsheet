"""df 1 and df 2 cells of cells.tsv: closed forms (df 2: EXP(-1.5 LN(2 + t t)), statistics.md) and LT1 (Loader)
for comparison; misses of LT1e on the hold-outs by block.  usage: s21_small.py"""
import math


import tdlib as TD

L, st = TD.LDB, TD.st
old = TD.load_old()
for n in (1.0, 2.0):
    rr = [r for r in old if r["n"] == n]
    forms = {"CR": lambda r: TD.exact_pdf(r["t"], n), "LT1(exact C)": lambda r: TD.lt1(r["t"], n, 0.0)}
    if n == 2:
        forms["EXP(-1.5 LN(2+tt))"] = lambda r: TD.exp_v6(L(st(L(-1.5) * L(TD.LN(st(L(2) + L(st(L(r["t"]) ** 2))))))))
    else:
        forms["1/pi/(1+tt)"] = lambda r: st(L(1 / math.pi) / L(st(L(1) + L(st(L(r["t"]) ** 2)))))
        forms["1/(pi*(1+tt))"] = lambda r: st(L(1) / L(st(L(math.pi) * L(st(L(1) + L(st(L(r["t"]) ** 2)))))))
        forms["1/(pi+pi*tt)"] = lambda r: st(L(1) / L(st(L(math.pi) + L(st(L(math.pi) * L(st(L(r["t"]) ** 2)))))))
    for k, f in forms.items():
        print(f"df {n:g} {k}: {TD.fmt(TD.tally([(f(r), r['excel']) for r in rr]))}")

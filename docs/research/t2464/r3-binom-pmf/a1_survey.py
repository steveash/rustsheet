"""Survey: Excel's BINOM/NEGBINOM pmf cells against the exact value, by region; and the first Loader models."""
import collections
import sys

import blib as B

rows = B.load_cells()
print("rows", len(rows), collections.Counter(r["fn"] for r in rows))


def region(r):
    if r["fn"] == "NEG":
        return "NEG"
    if r["k"] == 0:
        return "k0"
    if r["k"] == r["n"]:
        return "kn"
    return "int"


cr = collections.defaultdict(list)
for r in rows:
    e = float(B.exact_pmf(r["fn"], r["k"], r["n"], r["p"]))
    r["cr"] = e
    cr[region(r)].append((e, r["excel"]))
for g, v in sorted(cr.items()):
    print("CR", g, B.fmt(B.tally(v)))

models = {
    "LDe(R stir table, LF0, chain x)": dict(stir="R"),
    "stir ex": dict(stir="ex"),
    "stir ex, chain e": dict(stir="ex", chain="e"),
    "stir R, chain e": dict(stir="R", chain="e"),
    "stir R, LF1": dict(stir="R", lf="LF1"),
    "stir R, lf exact": dict(stir="R", lf="ex"),
    "stir R, order CsX": dict(stir="R", order="CX"),
    "stir R, bd0 r2 (ld e, v e, ph0)": dict(stir="R", bd0=dict(ld="e", v="e", ph=0)),
    "stir R, bd0 R cut 0.1": dict(stir="R", bd0=dict(cut=0.1, ld="R", v="x", ph=0)),
}
for name, o in models.items():
    res = collections.defaultdict(list)
    for r in rows:
        if r["fn"] != "BIN":
            continue
        m = B.loader_pmf(r["k"], r["n"], r["p"], o)
        res[region(r)].append((m, r["excel"]))
    print(name, {g: B.fmt(B.tally(v)) for g, v in sorted(res.items())})

"""Score the bgrat_half_ideal model on cells.tsv T tails (L side) in the bgrat branch, own dispatch and scorer.
usage: vcells.py"""

import math
from collections import Counter, defaultdict

import mpmath

import vlib

rows = vlib.load_cells()
sel = []
for r in rows:
    y, z = vlib.targs(r["t"], r["n"])
    r["y"], r["z"] = y, z
    r["br"] = vlib.branch(r["n"], y, z)
    if r["br"] == "bgrat":
        sel.append(r)
print("L-side T cells", len(rows), "branches", Counter(r["br"] for r in rows))
print("bgrat cells", len(sel))
keys = Counter((r["t"], r["n"], r["sc"]) for r in sel)
print("unique (t,n,scale)", len(keys), "unique (t,n)", len({(r["t"], r["n"]) for r in sel}))
print("cases", Counter(r["case"].split("/")[-1] for r in sel).most_common(20))

vlib.LN([r["y"] for r in sel])
variants = {
    "HX cut0.01": dict(),
    "always LN(y)": dict(lnmode="LN"),
    "cut 0.005": dict(cut=0.005),
    "cut 0.02": dict(cut=0.02),
    "CR erfc": dict(erfc="cr"),
    "z unrounded": dict(zround=False),
}
for name, kw in variants.items():
    kws = {k: v for k, v in kw.items() if k in ("cut", "zround")}
    if kw.get("lnmode") == "LN":
        ss = [math.sqrt(-(r["n"] / 2 - 0.25) * vlib.C[("LN", r["y"])]) for r in sel]
    else:
        ss = [vlib.needed_s(r["n"], r["y"], r["z"], **kws) for r in sel]
    vlib.ERFC(ss)
    pairs, hold, tune, uniq, byn = [], [], [], {}, defaultdict(list)
    caseh, casetune = [], []
    for r in sel:
        m = vlib.model_I(r["n"], r["y"], r["z"], **kw) * r["sc"]
        p = (m, r["excel"])
        pairs.append(p)
        (hold if vlib.myhold(r["case"] + r["cell"]) else tune).append(p)
        (caseh if vlib.myhold(r["case"]) else casetune).append(p)
        uniq[(r["t"], r["n"], r["sc"], r["excel"])] = p
        byn[r["n"]].append(p)
        r[name] = m
    print(
        f"{name}: all {vlib.score(pairs)} | cellhalf A {vlib.score(hold)} B {vlib.score(tune)} | "
        f"case-half A {vlib.score(caseh)} B {vlib.score(casetune)} | dedup {vlib.score(list(uniq.values()))}"
    )
    if name == "HX cut0.01":
        for n in sorted(byn):
            print("   n", n, vlib.score(byn[n]))
        print("   misses:")
        for r in sel:
            if r[name] != r["excel"]:
                print(
                    "    ",
                    r["fn"],
                    r["case"].split("/")[-1],
                    r["cell"],
                    repr(r["t"]),
                    r["n"],
                    r["excel"],
                    r[name],
                    f"{(r[name] - r['excel']) / math.ulp(r['excel']):+.0f} ulp",
                )
# baselines: CR at the double t
pairs = []
for r in sel:
    with mpmath.workdps(60):
        T, N = mpmath.mpf(r["t"]), mpmath.mpf(r["n"])
        Y = N / (N + T * T)
        cr = float(mpmath.betainc(0.5, N / 2, 1 - Y, 1, regularized=True) * r["sc"])
    pairs.append((cr, r["excel"]))
print("CR:", vlib.score(pairs))
# CR of I at the rounded y (no Excel pieces)
pairs = [(float(vlib.I_exact(r["n"] / 2, r["y"]) * r["sc"]), r["excel"]) for r in sel]
print("exact I at fl(y):", vlib.score(pairs))
vlib.save()
by = defaultdict(list)
for r in sel:
    by[r["case"].split("/")[-1]].append((r["HX cut0.01"], r["excel"]))
for k, v in by.items():
    print("case", k, vlib.score(v))
lo = [(r["HX cut0.01"], r["excel"]) for r in sel if r["z"] <= 0.01]
print("cells using log1p (z<=0.01):", vlib.score(lo))

"""T-3567 r5-fdf1: score the frozen fmodel (and the controls) on the hold-out t3567-r5-fdf1-hold, read once.
Columns: exact/w4/w15 per region, for fmodel (r >= 1 arguments), the s = n + mx control on the same kernel, and the
engine today (python bindings, the same formula text). Value cells only; error cells are listed."""
import json, math, sys
from collections import defaultdict
from pathlib import Path
import rustsheet as ic
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE)); sys.path.insert(0, str(HERE.parent / "r2-bratio"))
import fmodel  # noqa
import score_lib as sl  # noqa
NAME = "t3567-r5-fdf1-hold"
CASES = HERE.parents[2].parent / "eval/suites/probes-numeric/cases"
case = json.load(open(CASES / f"{NAME}.case.json"))["sheets"][0]["cells"]
out = json.load(open(CASES / f"{NAME}.oracle.json"))["sheets"]["Calc"]
meta = json.load(open(HERE / f"{NAME}.meta.json"))
m_ = ic.create("w", "en", "UTC")
keys = sorted(case, key=lambda k: (k[0], int(k[1:])))
for i, k in enumerate(keys):
    m_.update_cell_with_formula(0, i + 1, 1, case[k])
m_.evaluate()
eng = {k: m_.get_cell_value(0, i + 1, 1) for i, k in enumerate(keys)}
def num(v):
    return v if isinstance(v, (int, float)) and not isinstance(v, bool) else (v[-1] if isinstance(v, tuple) else v)
pts = [(x, math.trunc(m), math.trunc(n)) for _, _, x, m, n in meta]
res = {"fmodel": fmodel.run(pts), "s-control": fmodel.run(pts, fmodel.xy_s)}
g = defaultdict(list)
errs = []
for (k, reg, x, m, n), i in zip(meta, range(len(meta))):
    for col, side in (("A", 1), ("B", 0)):
        c = out.get(f"{col}{k}")
        if not c or c.get("t") != "n":
            errs.append((f"{col}{k}", case[f"{col}{k}"], c, num(eng[f"{col}{k}"])))
            continue
        e = c["v"]
        for name, r in res.items():
            v = r[i][side]
            for key in ((name, reg), (name, "all"), (name, "branch " + r[i][2] if name == "fmodel" else None)):
                if key[1]: g[key].append((v, e))
        ge = num(eng[f"{col}{k}"])
        for key in (("engine", reg), ("engine", "all")):
            g[key].append((float(ge) if isinstance(ge, (int, float)) else float("nan"), e))
        if name == "s-control" and res["fmodel"][i][2] == "bgrat":
            g[("engine", "branch bgrat")].append((float(ge) if isinstance(ge, (int, float)) else float("nan"), e))
print("| model | region | n | exact/w4/w15 |\n|---|---|---|---|")
for key in sorted(g, key=lambda t: (t[1].startswith("branch"), t[1], t[0])):
    n, ex, f, w = sl.tally(g[key])
    print(f"| {key[0]} | {key[1]} | {n} | {ex}/{f}/{w} |")
print(len(errs), "non-number cells")
for e in errs[:20]:
    print(e)

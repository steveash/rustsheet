"""LT1e: LT1 with each df's constant offset predicted by r3-loader-const's stirlerr table, e(h) - e(n/2) + c0, c0
fitted on the in-sample offsets only (lt1_offsets.json, dfs 3, 4, 5, 7, 10, 30).  Scores in-sample, old cells.tsv,
the first hold-out (post hoc: it was read before LT1e was formed) and hold2 (untouched until this run).
usage: s20_lt1e.py"""
import json

import mpmath
import numpy as np

import tdlib as TD
from s17_final import branch, score

ins_off = {float(k): v for k, v in json.loads((TD.HERE / "lt1_offsets.json").read_text()).items()}
e = TD.stirlerr_e()
d = {n: e[(n + 1) / 2] - e[n / 2] for n in ins_off}
c0 = float(np.mean([ins_off[n] - d[n] for n in ins_off]))
print(f"c0 = {c0:+.3f} (alpha = {2 * c0:+.2f} units per unit v); in-sample residuals "
      + " ".join(f"{n:g}:{ins_off[n] - d[n] - c0:+.2f}" for n in sorted(ins_off)))
TD.LT1_OFF.clear()
TD.LT1_OFF.update(TD.lt1e_offsets(c0, range(3, 56)))
cr = lambda r: float(mpmath.exp(TD.exact_logpdf(r["t"], r["n"])))  # noqa: E731
sets = {"in-sample": TD.load_probe("a1") + TD.load_probe("a2"), "old": [r for r in TD.load_old() if r["n"] >= 3],
        "hold(post-hoc)": TD.load_probe("hold"), "hold2": TD.load_probe("hold2")}
for nm, rows in sets.items():
    for r in rows:
        r["br"] = branch(r)
    score(f"{nm} LT1e", rows, lambda r: TD.lt1(r["t"], r["n"]))
    score(f"{nm} CR", rows, cr)
TD.save_exact()

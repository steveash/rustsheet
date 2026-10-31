"""T-2512 r3-bgrat: the bup sum Excel implies on the L140 bup+bgrat cells, w_xl = Excel - D (probe -split; D is
Excel's own increment, so w_xl is known to about ulp(G)/2 + ulp(D)/2), against the model's w (r2-bratio bup,
'w4' arithmetic) and the exact w. Units: ulps of w. Per shape group: median and sd of (w_M0 - w_xl) and
(w_ex - w_xl)."""
from __future__ import annotations

import json
import pickle
import statistics as st
from collections import defaultdict
from pathlib import Path

import mpmath

import bscore as score

HERE = Path(__file__).resolve().parent
NAME = "t2512-r3-bgrat-split"
out = json.load(open(HERE.parents[3] / "eval/suites/probes-numeric/cases" / f"{NAME}.oracle.json"))["sheets"]["Calc"]
D = {}
for k, a, b0, x, case, cell in json.load(open(HERE / f"{NAME}.meta.json")):
    v = out.get(f"A{k}")
    if v and v.get("t") == "n":
        D[(a, b0, x)] = v["v"]
exl = pickle.load(open(HERE / "exact_l140.pkl", "rb"))
mpmath.mp.dps = 40
g = defaultdict(list)
for c in score.cells():
    if not c["br"].startswith("L140 bup+bgrat") or (c["a"], c["b"], c["x"]) not in D:
        continue
    r = c["row"]
    if c["fn"](c["g"]) != c["g"]:
        continue  # swapped cells: Excel's lower tail is a complement; skip
    dx = D[(c["a"], c["b"], c["x"])]
    wxl = mpmath.mpf(r["excel"]) - dx
    wex = exl[(c["a"], r["b"], c["x"])] - exl[(c["a"], c["b"], c["x"])]
    u = mpmath.mpf(2) ** (mpmath.floor(mpmath.log(abs(c["w"]), 2)) - 52)
    g[(r["case"].replace("t2464-", ""), round(r["a"], 1), round(r["b"], 2))].append(
        (float((c["w"] - wxl) / u), float((wex - wxl) / u), float(c["w"] / r["excel"])))
print("| case | a~ | b~ | n | w/G | w_M0 - w_xl median, sd | w_ex - w_xl median, sd |")
print("|---|---|---|---|---|---|---|")
for k in sorted(g):
    v = g[k]
    m, e, f = [t[0] for t in v], [t[1] for t in v], [t[2] for t in v]
    print(f"| {k[0]} | {k[1]} | {k[2]} | {len(v)} | {st.median(f):.3f} | {st.median(m):.2f}, {st.pstdev(m):.2f} | "
          f"{st.median(e):.2f}, {st.pstdev(e):.2f} |")

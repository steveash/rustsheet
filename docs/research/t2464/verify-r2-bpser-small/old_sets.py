"""GL53/glalgdiv (my implementation, verify.py) vs r2-bratio BEST on the OLD data sets (be, w2f, w4p1, x29),
which the worker did not probe or tune on. Data rows via r2-bratio score_lib.load (data only); dispatch, model and
scoring are verify.py's. Lower-side BETA.DIST rows only (side == lower, y = fl(1-x))."""
import json, sys
from collections import defaultdict, Counter
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent)); import verify as v
T = v.T
sys.path.insert(0, str(T / "r2-bratio"))
import score_lib as sl  # noqa
import bratio_excel as bx  # noqa

args = json.load(open(T / "r2-bratio" / "t2464-r2-bratio-gammaln.args.json"))
out = json.load(open(T / "r2-bratio" / "t2464-r2-bratio-gammaln.out.json"))["sheets"]["Calc"]
for i, a in enumerate(args):
    c = out.get(f"A{i + 1}")
    if c and c.get("t") == "n":
        v._GL.setdefault(a, float(c["v"]))
v.load_gl_probe()
rows = [r for r in sl.load(("be", "w2f", "w4p1", "x29")) if r["side"] == "lower" and r["y"] == 1.0 - r["x"]]
for r in rows:
    r["hold"] = v.hold(r["a"], r["b"])
v.prepare(rows)
print("rows", len(rows), "GAMMALN fallbacks (harness model)", len(v.GL_FALLBACK))
keep = [r for r in rows if r["d"][0] == "w_bpser"]
for r in keep:
    d = r["d"]
    r["m"] = v.model(d[2], d[3], d[4], d[1])
    lo, hi = min(d[2], d[3]), max(d[2], d[3])
    r["fb"] = any(q in v.GL_FALLBACK for q in ([lo, hi, v.sx("+", lo, hi)] if hi < 8 else [lo]))
res = bx.run([(r["a"], r["b"], r["x"], r["y"]) for r in keep], bx.BEST)
for r, q in zip(keep, res):
    r["base"] = q[0]
    r["blab"] = q[2]
print("label mismatch vs bx", sum(not r["blab"].startswith("w_bpser") for r in keep))
g = defaultdict(list)
for r in keep:
    sb = v.sub_branch(r["d"][2], r["d"][3])
    g[(sb, r["set"])].append(r)
    g[(sb, "ALL")].append(r)
for k in sorted(g):
    rs = g[k]
    print(k, "n", len(rs), "GL53 ex", sum(r["m"] == r["excel"] for r in rs), "BEST ex", sum(r["base"] == r["excel"] for r in rs),
          "| GL53 w15", sum(v.w15(r["m"], r["excel"]) for r in rs), "BEST w15", sum(v.w15(r["base"], r["excel"]) for r in rs),
          "| cells w/ fallback GL", sum(r["fb"] for r in rs),
          "| measured-GL only: GL53", sum(r["m"] == r["excel"] for r in rs if not r["fb"]), "BEST", sum(r["base"] == r["excel"] for r in rs if not r["fb"]), "of", sum(not r["fb"] for r in rs))
blk = defaultdict(list)
for r in keep:
    blk[(r["set"], r["a"], r["b"])].append(r)
print("\nlargest old shapes:")
for k, rs in sorted(blk.items(), key=lambda kv: -len(kv[1]))[:12]:
    print(k, v.sub_branch(k[1], k[2]), len(rs), "GL53", sum(r["m"] == r["excel"] for r in rs), "BEST", sum(r["base"] == r["excel"] for r in rs), "fb", sum(r["fb"] for r in rs))

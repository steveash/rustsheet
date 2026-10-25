"""Compare the worker's stored variant values (W3z64 baseline, L53/glalgdiv, GL53, GL53/glalgdiv) on w_bpser cells by
sub-branch x set x hold, using verify.py's own dispatch and hold flag (pickle values checked identical for GL53/glalgdiv)."""
import pickle, sys
from collections import defaultdict
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import verify as v
pk = pickle.load(open(v.W / "results_shapes_runs_tiny.pkl", "rb"))
g = defaultdict(lambda: defaultdict(int))
for i, r in enumerate(pk["rows"]):
    d = v.dispatch(r["a"], r["b"], r["x"])
    if d[0] != "w_bpser":
        continue
    sb = v.sub_branch(d[2], d[3])
    for key in [(sb, r["set"], "hold" if v.hold(r["a"], r["b"]) else "tune"), (sb, r["set"], "all")]:
        g[key]["n"] += 1
        for m, vals in pk["vals"].items():
            g[key][m] += vals[i] == r["excel"]
for k in sorted(g):
    print(k, dict(g[k]))

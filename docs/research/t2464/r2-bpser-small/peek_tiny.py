"""List the tiny (apser / fpser) probe cells: Excel, model (BEST), CR, errors in ulps."""
import pickle

import lib

d = pickle.load(open(lib.HERE / "results_shapes_runs_tiny.pkl", "rb"))
for r, lab, m in zip(d["rows"], d["labs"], d["vals"]["W3z64"]):
    if r["set"] != "tiny":
        continue
    e = r["excel"]
    print(f"{lab:14s} a={r['a']:<8g} b={r['b']:<9.3g} x={r['x']:<8g} excel={e!r:24} model={lib.ulps_signed(m, e):+9.1f} "
          f"cr={lib.ulps_signed(r['cr'], e):+9.1f}  rel(cr-e)={(r['cr'] - e) / e:+.2e}")

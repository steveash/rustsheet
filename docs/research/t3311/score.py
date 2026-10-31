"""T-3311: main, HEAD and the correctly rounded quantile on the fresh sweeps, overall and in the fit's region,
with the Gate A envelope (decision 0188 R2).

Engine values come from two `rustsheet-eval run` reports (report.json lists every cell that is not bit-exact
with its actual value; any other scored cell equals Excel's). Excel's value and the 50-digit reference come
from reference_<sweep>.json (reference.py). The region is where excel_newton_quantile applies, judged here
with mpmath: shape 1.5 to 7.5, lower-tail probability L from 1E-200 to 1.000000082740371E-10, the cdf at the
leading-factor root `exp((ln L + lnGamma(a) + ln a) / a)` within 1E-6 of L relative, and not within 5E-14
(T-2561's root, which comes first). Scores: bit-exact / within 4 ulp / within half a unit of Excel's 15th
significant digit; Gate A: within 15 digits of Excel or within 1E-9 relative of the reference (floor 1E-300).

Usage (repo root): uv run --no-project --with mpmath python docs/research/t3311/score.py MAIN_REPORT HEAD_REPORT [OUT]
Output: docs/research/t3311/OUT (default score.out). Committed: score_cut1e-6.out (the model frozen in 63d859826, the
cut the hold-out picks; fresh sweep 2 was filled after that freeze) and score_cut1e-8.out (the cut the branch holds
pending the A107 ruling). In-region rows use the 1E-6 region as defined above for both.
"""
import json
import math
import sys
from pathlib import Path

import mpmath as mp

mp.mp.dps = 40
HERE = Path(__file__).parent
SWEEPS = ["t3311-fresh-sweep", "t3311-fresh-sweep-2"]


def engine(report, case):
    data = json.load(open(report))
    out = {}
    for c in data["cases"]:
        if c["id"] == f"probes-numeric/{case}.case.json":
            for cell in c["cells"]:
                out[cell["cell"]] = cell["actual"].get("v") if cell["actual"]["t"] == "n" else cell["actual"]
    return out


def visible(x, e):
    return isinstance(x, float) and abs(x - e) <= 0.5 * 10 ** (math.floor(math.log10(abs(e))) - 14)


def in_region(a, low):
    if not (1.5 <= a <= 7.5 and 1e-200 <= low <= 1.000000082740371e-10):
        return False
    a, low = mp.mpf(a), mp.mpf(low)
    root = mp.exp((mp.log(low) + mp.loggamma(a) + mp.log(a)) / a)
    miss = abs(mp.gammainc(a, 0, root, regularized=True) / low - 1)
    return 5e-14 < miss <= 1e-6


def main():
    main_report, head_report = sys.argv[1], sys.argv[2]
    out_name = sys.argv[3] if len(sys.argv) > 3 else "score.out"
    lines = []
    for sweep in SWEEPS:
        rows = json.load(open(HERE / f"reference_{sweep.replace('t3311-', '')}.json"))
        actual = {"main": engine(main_report, sweep), "HEAD": engine(head_report, sweep)}
        groups = {"all": [], "region": []}
        for r in rows:
            cell = r["key"].split()[0]
            scale = float(r["scale"])
            excel = float(r["excel"]) * scale
            true = mp.mpf(r["true"]) * scale
            vals = {name: actual[name].get(cell, excel) for name in actual}
            vals["correctly rounded"] = float(mp.mpf(r["cr"]) * scale)
            point = (r["key"], excel, true, vals)
            groups["all"].append(point)
            if in_region(float(r["a"]), float(r["L"])):
                groups["region"].append(point)
        lines.append(f"== {sweep}")
        for g, pts in groups.items():
            lines.append(f"  {g} ({len(pts)} cells): exact / within 4 ulp / within 15 digits / Gate A")
            for name in ["HEAD", "main", "correctly rounded"]:
                s = [0, 0, 0, 0]
                outside = []
                for key, e, t, vals in pts:
                    x = vals[name]
                    if not isinstance(x, float):
                        outside.append(key)
                        continue
                    s[0] += x == e
                    s[1] += abs(x - e) <= 4 * math.ulp(e)
                    s[2] += visible(x, e)
                    ok = visible(x, e) or abs(mp.mpf(x) - t) <= max(mp.mpf("1e-9") * abs(t), mp.mpf("1e-300"))
                    s[3] += ok
                    if not ok:
                        outside.append(key)
                lines.append(f"    {name:18} {s[0]:4} {s[1]:4} {s[2]:4} {s[3]:4}" + (f"  outside Gate A: {outside}" if outside else ""))
        changed = [(k, vals["main"], vals["HEAD"], e) for k, e, t, vals in groups["all"] if vals["main"] != vals["HEAD"]]
        lines.append(f"  main -> HEAD changed cells: {len(changed)}")
        for k, m, h, e in changed:
            um, uh = (m - e) / math.ulp(e), (h - e) / math.ulp(e)
            lines.append(f"    {k}: main {um:+.0f} ulp, HEAD {uh:+.0f} ulp")
    text = "\n".join(lines) + "\n"
    (HERE / out_name).write_text(text)
    print(text)


if __name__ == "__main__":
    main()

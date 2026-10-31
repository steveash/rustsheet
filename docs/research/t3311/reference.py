"""T-3311: 50-digit mpmath reference quantiles for the hold-out split points and the fresh sweeps.

Each point is a lower-tail gamma quantile at scale 1: shape a, lower-tail probability L (the double Excel
works with: p for GAMMA.INV/CHISQ.INV, 1 - p for CHISQ.INV.RT, formula literals cut to 15 significant digits),
and Excel's value at scale 1 (CHISQ.* values halved, GAMMA.INV divided by its scale). The reference is the
root of P(a, x) = L by Newton steps at 60 digits, printed to 50; `cr` is it rounded to a double (the
correctly rounded candidate of decision 0188 R4.1).

Inputs:
- split_points.json (this directory): the 153 distinct (shape, p) points of the R4.1 split, written by
  T-3311's collector from probes-numeric/t3311-gamma-inv-lower-tail-cdf (sheets W and S) and the literal
  GAMMA.INV/GAMMAINV/CHISQ.INV/CHISQ.INV.RT/CHIINV cells of gen-m9, probes and probes-numeric at shapes 1 to
  20 (right tail only at 1 - p <= 1E-5).
- each fresh sweep case under eval/suites/probes-numeric/cases (t3311-fresh-sweep*).

Usage (repo root): uv run --no-project --with mpmath python docs/research/t3311/reference.py
Output: docs/research/t3311/reference_<name>.json, one row per point: key, a, L, scale, excel (at scale 1),
true (50 digits, at scale 1), cr.
"""
import json
import re
import sys
from decimal import ROUND_DOWN, Decimal
from pathlib import Path

import mpmath as mp

mp.mp.dps = 60
HERE = Path(__file__).parent
CASES = Path("eval/suites/probes-numeric/cases")


def cut15(text):
    d = Decimal(text)
    return float(d.quantize(Decimal(1).scaleb(d.adjusted() - 14), rounding=ROUND_DOWN))


def quantile(a, low, start):
    a, low = mp.mpf(a), mp.mpf(low)
    x = mp.mpf(start) if start > 0 else mp.exp((mp.log(low) + mp.loggamma(a + 1)) / a)
    for _ in range(300):
        f = mp.gammainc(a, 0, x, regularized=True) - low
        step = f / mp.exp((a - 1) * mp.log(x) - x - mp.loggamma(a))
        nx = x - step
        if nx <= 0:
            nx = x / 2
        if abs(nx - x) <= abs(x) * mp.mpf(10) ** -55:
            return nx
        x = nx
    raise RuntimeError(f"no convergence at a={a} L={low}")


def row(key, a, low, excel, scale=1.0):
    t = quantile(a, low, excel)
    return {"key": key, "a": repr(a), "L": repr(low), "scale": repr(scale), "excel": repr(excel), "true": mp.nstr(t, 50), "cr": repr(float(t))}


def sweep_points(stem):
    case = json.load(open(CASES / f"{stem}.case.json"))
    oracle = json.load(open(CASES / f"{stem}.oracle.json"))["sheets"]
    for sheet in case["sheets"]:
        for cell, formula in sheet["cells"].items():
            m = re.fullmatch(r"=(GAMMA\.INV|CHISQ\.INV\.RT|CHISQ\.INV)\((1-)?([^,]+),([^,]+)(?:,([^)]+))?\)", formula)
            value = oracle[sheet["name"]].get(cell, {})
            if not m or value.get("t") != "n":
                continue
            fn, one, p, a, s = m.groups()
            p = cut15(p)
            if one:
                p = 1.0 - p  # Excel's double for 1-q
            if fn == "GAMMA.INV":
                shape, scale = cut15(a), cut15(s)
            else:
                shape, scale = int(cut15(a)) / 2, 2.0
            low = 1.0 - p if fn == "CHISQ.INV.RT" else p
            yield f"{cell} {formula}", shape, low, value["v"] / scale, scale


def main():
    names = sys.argv[1:] or ["split"] + sorted(p.name[: -len(".case.json")] for p in CASES.glob("t3311-fresh-sweep*.case.json"))
    for name in names:
        if name == "split":
            pts = [(d["key"], float(d["a"]), float(d["p"]), float(d["e"])) for d in json.load(open(HERE / "split_points.json"))]
        else:
            pts = list(sweep_points(name))
        rows = [row(*pt) for pt in pts]
        (HERE / f"reference_{name.replace('t3311-', '')}.json").write_text(json.dumps(rows, indent=0) + "\n")
        print(name, len(rows))


if __name__ == "__main__":
    main()

"""T-3571 r5-bfrac: build probes t3571-r5-bfrac-tune and t3571-r5-bfrac-hold (probes-numeric), filled together.

Each run is 96 consecutive doubles in x from an irregular start x0 = mu + z*sd (7 significant digits; mu, sd the beta
mean and sd) at z = -7, -4, -1, on shapes where TOMS 708 bratio takes bfrac (dispatch(); a run with any basym point is dropped). Row k:
A = BETA.DIST(x, a, b, TRUE), B = BETA.DIST(x, a, b, FALSE). The z levels change how many fraction terms bfrac takes
(6 to 40 here), so a fraction-arithmetic noise and a prefactor noise scale differently across them.
Meta: [[row, a, b, z, x], ...].   usage: mkprobe.py [--fresh] [--write]   (--fresh: t3571-r5-bfrac-fresh, built after the model freeze)"""
from __future__ import annotations
import importlib.util, json, math, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
CASES = HERE.parents[3] / "eval" / "suites" / "probes-numeric" / "cases"
_spec = importlib.util.spec_from_file_location("w2probes", HERE.parent / "w2-forensics" / "make_probes.py")
w2p = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(w2p)
TUNE = [(40.0, 45.0), (100.0, 120.0), (1000.0, 1200.0), (300.0, 60.0), (45.0, 2000.0), (60.5, 75.25)]
HOLD = [(42.5, 50.0), (150.0, 170.5), (700.0, 90.0), (55.0, 400.0), (2500.0, 3000.0), (80.0, 81.0)]
ZS = (-7.0, -4.0, -1.0)
RUN = 96

def dispatch(a, b, x, y):
    """TOMS 708 bratio's path for a, b > 1 (R's toms708.c): 'bfrac', 'basym' or 'other'."""
    lam = (a + b) * y - b if a > b else a - (a + b) * x
    a0, b0 = (a, b) if lam >= 0 else (b, a)
    lam = abs(lam)
    if b0 < 40: return 'other'
    if a0 > b0: return 'bfrac' if (b0 <= 100 or lam > b0 * 0.03) else 'basym'
    if a0 <= 100 or lam > a0 * 0.03: return 'bfrac'
    return 'basym'

def build(shapes):
    cells, meta = {}, []
    for a, b in shapes:
        mu = a / (a + b); sd = math.sqrt(mu * (1 - mu) / (a + b + 1))
        for z in ZS:
            x0 = mu + z * sd
            if x0 <= 1e-3: continue
            x = float(f"{x0:.7g}")
            xs = [x]
            for _ in range(RUN - 1): xs.append(math.nextafter(xs[-1], 1.0))
            if any(dispatch(a, b, v, 1 - v) != 'bfrac' for v in xs): continue  # basym there
            for x in xs:
                k = len(meta) + 1
                cells[f"A{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b)},TRUE)"
                cells[f"B{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b)},FALSE)"
                meta.append([k, a, b, z, x])
    return cells, meta

def build_fresh(seed=35719, count=1200):
    """Fresh sweep (built after model.py's freeze): log-uniform a, b in [40, 5000] with 1-6 significant digits, x at
    z uniform in [-8, -0.2] sd (15 significant digits), one point per shape; bfrac points only."""
    import random
    rnd = random.Random(seed)
    cells, meta = {}, []
    while len(meta) < count:
        a = float(f"{math.exp(rnd.uniform(math.log(40), math.log(5000))):.{rnd.randint(1, 6)}g}")
        b = float(f"{math.exp(rnd.uniform(math.log(40), math.log(5000))):.{rnd.randint(1, 6)}g}")
        mu = a / (a + b); sd = math.sqrt(mu * (1 - mu) / (a + b + 1)); z = rnd.uniform(-8.0, -0.2)
        x = float(f"{mu + z * sd:.15g}")
        if x <= 1e-3 or dispatch(a, b, x, 1 - x) != 'bfrac': continue
        k = len(meta) + 1
        cells[f"A{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b)},TRUE)"
        cells[f"B{k}"] = f"=BETA.DIST({w2p.num(x)},{w2p.num(a)},{w2p.num(b)},FALSE)"
        meta.append([k, a, b, round(z, 3), x])
    return cells, meta

def main():
    sets = (("t3571-r5-bfrac-tune", TUNE), ("t3571-r5-bfrac-hold", HOLD))
    if "--fresh" in sys.argv: sets = (("t3571-r5-bfrac-fresh", None),)
    for name, shapes in sets:
        cells, meta = build(shapes) if shapes else build_fresh()
        case = {"format": "rustsheet-case/v1",
                "description": (f"T-3571 r5-bfrac: BETA.DIST cdf and pdf on runs of 96 consecutive doubles in TOMS 708's "
                                f"bfrac region at z = -7, -4, -1 sd ({'in-sample' if 'tune' in name else 'hold-out'}; "
                                if shapes else "T-3571 r5-bfrac fresh sweep (built after the model freeze): BETA.DIST cdf and "
                                "pdf at log-uniform (a, b) in [40, 5000], one x per shape in TOMS 708's bfrac region (")
                + "docs/research/t2464/r5-bfrac)",
                "tags": ["M9", "probe", "statistics", "T-3571", "incomplete-beta"],
                "sheets": [{"name": "Calc", "cells": cells}], "expected": {}}
        assert len(cells) <= 16384
        print(name, len(meta), "points", len(cells), "cells")
        if "--write" in sys.argv:
            (CASES / f"{name}.case.json").write_text(json.dumps(case, indent=1) + "\n")
            json.dump(meta, open(HERE / f"{name}.meta.json", "w"))

if __name__ == "__main__":
    main()

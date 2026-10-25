"""Positive controls for vlib: EXP V6 against the engine's EXP on doubles, log1p series against r2-log1p's
function, exact density against scipy-free closed form at simple points, scorer sanity."""
import random
import sys
from pathlib import Path

import vlib as V

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "r2-log1p"))
import excel_log1p as XL  # noqa: E402

random.seed(7)
es = [random.uniform(-700, 700) for _ in range(1500)] + [random.uniform(-3, 3) for _ in range(1500)]
ref = V.XM.excel("EXP", es)
bad = sum(V.exp_v6(e) != y for e, y in zip(es, ref))
print(f"EXP V6 (mine, double args) vs engine EXP: {bad} mismatches of {len(es)}")
rs = [random.uniform(0, 0.375) for _ in range(3000)]
bad = sum(V.log1p_series(r) != XL.log1p_series(r) for r in rs)
print(f"log1p series vs r2-log1p: {bad} mismatches of {len(rs)}")
# F(1; 2, 2) = 1/(1+1)^2 = 0.25 ; F(x; 2, n) = (1 + 2x/n)^(-(n/2+1))
print("exact F(1;2,2)", V.f_exact(1.0, 2, 2), " F(0.5;2,4)", V.f_exact(0.5, 2, 4), float((1 + 0.25) ** -3))
print("w15 checks", V.w15(1.23456789012345e-3, 1.23456789012346e-3), V.w15(1.2345678901234e-3 + 1e-18, 1.2345678901234e-3))

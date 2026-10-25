"""hix probe: per row the implied offset q = ln(B) - 1024*lnx must be one constant for the right lnx.
Candidates: Excel LN(x); this worker's log1p series at -y (y = fl(1-x)) for y <= 0.375; CR ln x.  Also C
(b = 1.5, general density) as a second column.  Usage: hix_check.py"""
import math

import mpmath

import cands
import excel_log1p as XL
import make_probe_hi as MH
import r2lib as R

xs = MH.sets()
o = R.read_oracle("hix")
LNx = R.xl("LN", xs)
C = {"LN(x)": LNx, "series(-fl(1-x))": [XL.log1p_series(-(1.0 - x)) if 1 - x <= 0.375 else math.nan for x in xs],
     "CR ln x": [float(mpmath.log(R.mpf(x))) for x in xs]}
for col, a1 in (("B", 1024), ("C", 1024)):
    vals = [o.get(f"{col}{i + 1}") for i in range(len(xs))]
    for nm, lx in C.items():
        qs = [float(mpmath.log(R.mpf(v)) - a1 * R.mpf(l)) if isinstance(v, float) and v > 0 and not math.isnan(l)
              else math.nan for v, l in zip(vals, lx)]
        # col C has the (b-1)*ln y term too: remove it with the model ln y
        if col == "C":
            ly = [XL.ln1m(x) for x in xs]
            qs = [q - 0.5 * float(R.mpf(l)) for q, l in zip(qs, ly)]
        good = sorted(q for q in qs if not math.isnan(q))
        med = good[len(good) // 2]
        tol = 0.5 * a1 * 2.0**-55
        ok = sum(abs(q - med) <= tol for q in good)
        print(f"col {col} lnx={nm:18s} rows {len(good)} at the common offset {ok}  (median q {med:+.15f})")

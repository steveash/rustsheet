# T-1778: Excel EXP emulation (round 4 exp-exact)

- **Model** (`exp_ref.py`): t = RN64(a*RN64(log2 e)), n = floor(t), f = RN64(t - n), u = RN64(2^f - 1),
  w = RN64(1 + u), EXP = RN53(w*2^n), with subnormal results kept. It matches 153,643 of the 153,655 nonzero
  EXP cells.
- **The 12 residual cells** (`mis12.txt`) need Excel's F2XM1 to differ from the correctly rounded 2^f - 1 by 1-2
  ulp, all with f > 0.59.
- **The F2XM1 map** (`kmyst-erfc-r5expmap1/2`, scored by `score_expmap.py`) shows the error depends on frac(t),
  is deterministic, and is not a constant offset within a cluster.
- **Data.** `build_data.py` rebuilds expdata.txt from the probe sidecars (the paths inside may point at the
  session scratch directory). The C files are the x87 emulation and the search tools.

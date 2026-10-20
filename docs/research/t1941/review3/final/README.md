# Reviewer 3 final K2 scripts (2026-10-20 pause)

Run these from the repo root. They import `spec` from `docs/research/t1941/review3`. First replace the literal
`SCRATCH/` with a local directory. `allscore.py` writes `allXY.npy` (every unique x in (2,3]), which `cand8.py`
and `cand9.py` read. `cand*_PQ.npy` files are arrays [P(10), Q(9)+pad]. The findings are in HANDOFF2 §12.1.

| Group | Scripts |
|---|---|
| Structure | `near2mask.py`, `near2.py`, `near2b.py`, `ops.py`, `opmask.py`, `mix2.py`, `transition.py` |
| Coefficients | `clusterbias.py` (bias curve), `cand8.py` (integer coordinate search), `cand9.py` (near-2-constrained scan), `remez.py`, `allscore.py` |
| Residual | `c3scan.py`, `c3sub.py`, `c3all.py`, `c3table.py` (per-run N.s3 offset); `s3line.py`, `phaseM.py` (per-op line phases) |
| Older (superseded; R5 hack and parity rules) | `cx10.py`, `cx12.py`, `cx13.py`, `layer3.py`, `lateafter.py`, `drift.py`, `boundary.py` (boundary imports `fitrule` from drift via `SCRATCH`) |

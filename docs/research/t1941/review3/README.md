# T-1941 third review scripts (Fable 5.1, 2026-10-20)

Companion to `../HANDOFF.md` section 12.
- Run from the repo root with `python3 -I docs/research/t1941/review3/<script>`.
- Needs numpy only: x87 longdouble, `astype(float64)` after every stored op.
- All inputs are committed oracle cases. No oracle calls.
- The coefficient set is grid_a exactly as committed in `../w2/grid_a.txt`, with no needle correction (see 12.4).

| Script | What it computes |
|---|---|
| `lines.py X` | base stored-Horner line frequencies (sum, product, triangle) of every grid_a stage at x, plus the first-rounding line a two-part head would have |
| `spec.py`, `spec2.py MINLEN cases...` | residual spectra of Excel - exact rational and sim - exact rational per long run; power at the N.S4 / D.S3 / 8*p3 / 8*q2 frequencies; Excel-only and sim-only lines |
| `freq.py MINLEN cases...` | sub-bin (1e-5 cycle/step) frequency, power and phase of the N.S4 and D.S3 sum lines, Excel vs sim, with the two-part-head prediction f_MC |
| `flips.py CORR cases...` | per-point admissible (dN, dD) shifts at the (4,3) hand-off with the late chain exact; minimal-shift histogram; need-shift rate by sawtooth phase octile |
| `clusters.py cases...` | per-512-cluster best constant integer shift; sign mixing; phase lock over all stages; the same for synthetic perturbed truths |
| `keeptest.py cases...` | conditional agreement test: where an unstored-op variant differs from the stored chain, which one does Excel follow |
| `p3scan.py cases...` | per-run 1-D scans of p2, p3, p4, q2, q3 |
| `dgridlike.py cases...` | dgrid-style per-cluster fractional pre-rounding offset scan at (4,3) (80-bit head + offset), real vs an all-stored synthetic truth |
| `offm.c` / `offm.py [truth.pkl|ctl:...] cases...` | (W2) per-run hand-off offset meter of HANDOFF 12.3 step 1; build `liboffm.so` next to it. Control `ctl:p4:+2` on r113 gives 1.000 at dN +2 |

Long consecutive GAMMALN.PRECISE runs in (2,3] used:
- r701, r112, p401, r702, r610, r901 (two runs), r703, r113, r704, r611, r705, v401a, v404a, v407a;
- 8,192 or 16,384 consecutive doubles each.

The q2xx dense cases are 512-double clusters and are too short for sub-bin frequency work.

## Added with the follow-up (HANDOFF 12.6)
| Script | What it computes |
|---|---|
| `meter.py [--control] [--stage kN kD] [--mode pre\|post] cases...` | the tie-class per-anchor offset meter: one smooth offset per chain, injected before the product's quantisation (`pre`) or on the constant (`post`) at the given stages, scanned in 1/32 ulp of the stored sum over +-3.5 by coordinate ascent; `--control` plants known moves and prints the expected readings |
| `meter_keep.py cases...` | the same meter on top of head models with the stage-4 (N) / stage-3 (D) products, or all head products, left unstored |
| `latescan.py cases...` | each late coefficient moved in whole units of its stored sum's grid, m in [-6,6], scored by the (4,3)-pre meter's best rate per run |
| `ties.py cases...` | direct test of how Excel resolves each position class (exact tie, 1/4, 3/4, 0) of the hand-off sums against ties-to-even and the true side of the unquantised product |

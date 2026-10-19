# T-1941 W-A (W2): K2 plateau reader and early-stage anchor phases

## Settled
1. **grid_a's late plateau bits are Excel's.**
   - Excel - sim(grid_a) has no staircase, only a smooth ±0.3-ulp function error.
   - The plateau bit-reader (`iterbits.py`, BIG=1) passes its positive control: planted N9 +3072, N8 -64 and D7 +128
     are recovered, 100% exact. It must be restricted to wide-window stages; with the narrow ones free it lands in
     wrong minima.
2. **Excel's function on (2,3]** = R_grid_a + smooth δ (`target.py`, Legendre deg 4-12, chi2/dof 0.85-0.93).
   - The joint phase+function solve passes its control with the full 28-tier unwrap. Coarse tiers caused the earlier
     "conflict", now retracted.
   - On real data the ladder phases and Excel's function are consistent in (9,8) raw x.
3. **efC** (`../efC.txt`, `earlyfit2.py`): grid_a's late coefficients, with the early p0-p4/q0-q2 fitted to Excel's
   function while the ladder stage values are held.
   - Staircase flat (-0.00..+0.06 ulp); ladder rms 0.010-0.016.
   - 35.2% exact. The remaining error is in the early stage values.

## Open
4. **Early-stage anchor-phase solve** (`nullb.py`, `phasecore.py`, `grid3.py`, `ridge.py`, `climb.py`).
   - Inputs: multiplicand lines of N s4/s5 and D s3/s4 at 8 dyadic anchors (e60x/e80x) plus 12 non-dyadic allX cases.
   - Synthetic control (truth = efC + p1 -669 / q0 -3026 along the null space):
     - the phases pin everything except one ridge, Δq0 ≈ 3.2·Δp1;
     - climbing along it reaches 98.7% exact, 200-430 ulps from the truth;
     - closing it needs another independent family.
   - s1-s3 rows fail the control. The triangle and doublet laws have not been tried for them.
5. **Real data contradicts the model at these lines.**
   - Best coherence 0.42-0.49, or 0.55 with p5 and q3-q5 free (synthetic: 0.99).
   - Single-coefficient scans over p1-p5 and q0-q4 peak at 0.23-0.35.
   - Per-anchor mean phases at efC:
     - N s5: +0.47 / -0.46 / -0.36 / -0.28 / +0.25 at 2.25 / 2.375 / 2.5 / 2.625 / 2.875;
     - D s4: -0.47 / +0.50 / -0.41 at 2.375 / 2.5 / 2.75;
     - both about 0 at 2.125.
   - So Excel's N s4/s5 and D s3/s4 product lines do NOT follow the stored-product sawtooth law relative to any
     reachable (9,8) raw-x set. Synthetic data from the model does follow it, so the extraction is sound.
   - Hypotheses:
     - a different product formation (64-bit unstored, or a different multiplicand or chain split);
     - a triangle or doublet law not yet applied;
     - rho = 1/2 where the S grid sits one binade up.
   - Suggested next round: per-line law checks on real data, using W4's single-stage synthetic per line plus
     triangle/doublet variants for N s4/s5 and D s3/s4 at 2.25-2.75.

## Files
- `bits.py`, `iterbits.py`: plateau bit-reader.
- `target.py`: Excel's smooth function target.
- `earlyfit2.py`: efC.
- `nullb.py`: function+ladder null basis.
- `phasecore.py`, `grid3.py`, `ridge.py`, `climb.py`: anchor-phase solve and control.
- Not committed: `anchtab.py` (phase table via w2's fixedref.build_table) and `fsE.py` (joint ladder+function solve
  via fixedsolve). Both are described in W2's report.
- Dependencies: `../ysearch.py`, libpoly.so with a gbase entry (y = fl((x-2)*fl(N/D)); `../poly.c` has gcount only),
  `review/k23lib.py`, and npy data built from rows.txt.

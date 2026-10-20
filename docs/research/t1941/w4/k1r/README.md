# T-1941 W4 K1 tools (k1r/): per-run readings, near-null lattice search, local/cluster fits, keep-mask and tie meter

Built on review2's gsim/lib2. AN2 is the base everywhere, and exact match is always the verifier.

Setup:
- Set SCR (k1lib does) to the directory holding `g_rows.txt` (from `g_rows.py`).
- `k1lib.R2` points at review2/; edit it to your checkout.

## Tools
- `k1lib.py`: dense runs from g_rows (chunks of 8192), the 1-D nudge `scan` (1/32 steps, refined), traces.
- `readex.py STAGES PREFIX TAG`: per-run refined readings, e.g. `D.P:4,N.P:5`, with the stage grids.
- `nulllat.py LAM`: BKZ-12 on [I | lam·J] over N0..N7, D1..D7. It gives short integer moves (1-4 ulps) with a function
  change of at most 0.002-0.012 G-ulp.
- `latclimb.py`, `lathop.py`: exact-count ascent and basin hopping in that lattice basis.
  - `lathop.py all NV PER HOPS excel OUT` scores a stratified sample of [1,2) (PER points per tenth). Env: FRZ = frozen
    vector indices, KICK.
- `holdout.py PKL "grp;grp"`: AN2 vs a lattice move on held-out run groups.
- `localfit.py RUN SEED`, `clusterfit.py RUNS SEED`: W2-style null-direction segment search (k = 1..4) on one run or a
  cluster.
  - clusterfit env: YLO/YHI (y-window), ST (stride), ROUNDS, START (pkl), OUT.
- `pins.py`, `intervals.py`: stage pinning across fits; constant intervals c in s - p +- u/2, intersected.
- `gsimk.c` / `gsk.py`: K1 simulator with per-op keep masks (unstored product/sum). `clusterfitk.py` takes env KN/KD.
- `oprep.py SRC RUNS YLO YHI STRIDE`: per-op repair test (net repaired - broken, one op made exact).
- `tiemeter.py CH K RUN [delta]`: review 3's tie-class triple, with a planted-offset calibration.
- `d2test.py FIXED FREE`: a fixed move plus exact LS function compensation, scored by tenth on all of [1,2).

## Results (2026-10-19/20), all with controls
1. **Function sensitivity per ulp** (max |dG| in G-ulps): D1-D5 about 0.7 each (D2 1.43), N0 0.70, N1/N2 0.36, N3 0.18,
   N4 0.089, N5 0.022, N6 0.0055, N7 0.0027, D6 0.18, D7 0.022.
   - So D1-D5 are each pinned to about 0.1 ulp by the function.
   - Review 2's "+-5e3 per-coefficient unwrap box" is therefore infeasible: Excel - AN2 must be a near-null integer
     combination.
   - Its suggested control (D3 +437, D4 -212) changes G by about 20 ulps, and AN2 is 0% exact against it.
2. **Lattice basin hopping.**
   - Control: AN2 + k·V, AN2 0.478 exact against it -> 0.9993 recovered; only top-only vectors differ.
   - Excel, P1 only: 0.6244 -> 0.6370 (lat1). It holds on held-out y1xx/r7xx/e10, but lat1 is worse on all of [1,2):
     0.7345 -> 0.7272, from the top move (N7 -52, N6 +23).
   - Stratified whole-interval objective with top vectors frozen: AN2 is a LOCAL MAXIMUM (0.7067, 80 hops; 15 vectors
     give 0.7164 only by trading 1.9x).
3. **Local and cluster fits** (W2 route) are aliases.
   - Single runs reach 0.999 and score 0-0.34 at their neighbours. A 2-run pair is valid over +-0.005 in y.
   - Ladder anchors: 1.875 and 1.7718 reach 0.9995; V1 1.9253 stalls at 0.845; the 0.02 window stalls at 0.913.
   - A far-start pin check at 1.7718 agrees on N S6/P7/S7 and D S4-S7. The implied D2 = AN2 + 350 (the same at three
     non-dyadic sites) is rejected globally: function-compensated, all of [1,2) goes 0.674 -> 0.289.
   - What holds: N0 and D1 equal AN2 (<=2-4 ulps) under every local fit.
4. **Unstored-op hypothesis.**
   - The repair test on AN2 flags D.add4 and N.add5 (the stages review 2's readings flag).
   - Stored-fit repair tests show no positive op, and refits with those ops unstored are worse (0.741 / 0.648 / 0.774 vs
     stored 0.913 / 0.845 / 0.999). Rejected: the hot ops localise AN2's error.
5. **Tie-class meter (review 3).**
   - Calibration at D.S4: the triple is (0, 1, 0) at offset 0 and (0.50, 0.50, 0) for every nonzero offset. It reads
     sign only on K1.
   - Stage grid ratios u(S)/u(P): N S1 4-8, S2 4-8, S3 2-4, S4 2-4, S5 2-4, S6 1-4, S7 1-2; D S1 8-16, S2 4-8, S3 0.5-2,
     S4 4, S5 0.5-2, S6 2, S7 1.

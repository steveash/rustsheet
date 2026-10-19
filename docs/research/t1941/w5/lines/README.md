# W5 part 2: near-1 kernel line analysis with W4's triangle law (T-1941)

Data: the 92 far runs (|x-1| > 0.02, at least 2,048 consecutive doubles) in [0.7, 1.5). The residual per run is
(B - c5 exact function)/ulp(B). The candidate is c5_f99t, simulated exactly (`../sc.c`).

## 1. The "unexplained" far lines are tie lines of c5's own sum stages
`peaks.py` finds 670 strong data peaks; 71 of them have no counterpart in c5's simulation. They form three tracks:
- T1: f = 0.3134 at x = 0.7413, rising to 0.3896 at x = 0.976.
- T2: f = 0.3515 at x = 0.701, falling to 0.3396 at x = 0.784.
- T3: f = 0.467 at x = 1.347, falling to 0.406 at x = 1.481.

No c5 stage under the sawtooth law, and no 2x harmonic or sum/difference of two predicted lines, follows any
of them (`tracks.py`). The full-track test against every intermediate (`altcomp.py`) gives:

| track | stage | law | max abs(df) over the track |
|---|---|---|---|
| T1 | D.S3 = fl(D.P3 + D3) | f = S'·h/(2·ulp S) | 6.7e-5 |
| T2 | N.S2 = fl(N.P2 + N2) | f = S'·h/(2·ulp S) | 6.6e-5 |
| T3 | D.S3, in the x > 1.34 region | same law | 1.3e-3 |

This is W4's triangle law, F = S'/2^(eS+1). In both regions the sum lands one binade above its product:
- D.S3: S3 enters [1,2) at x = 0.7413 while P3 stays below 1 until 0.9922.
- N.S2: same picture over x = 0.70-0.78.

So c5's stage values are right at the line-frequency level. The lines are missing from c5's simulation for a
different reason.

## 2. Mechanism: round-half-even ties. Reading coefficient bits
- When c sits exactly on P's grid (P = n·ulp P), S = P + c is exact for even n + c' and a tie for odd. A tie
  rounds to even through the next bit, so the error has period 4 ulp(P), which is f = S'h/(2 ulp S).
- When c is half a P-ulp off the grid (c5's D3 and N2, odd mantissa), S errs by ±1/4 ulp(S) with period 2,
  which is f = S'h/ulp S. That is what c5 simulates (`amp.py`; the sim's S3 errors are exactly ±0.25).
- **Excel's D3 and N2 have even last bits. c5's are odd.** (`parity_test.py`)
  - D3 -> D3-1 makes the D.S3 line appear in simulation at the data's amplitude: 233 against 229.
  - N2 -> N2+1 does the same for N.S2: 185 against 198.
  - Near-1 exact stays at 0.7455 for both.
- **Survey of every one-binade-up sum stage** (`survey.py`; amplitudes in ulps of B):

  | stage, P binade | c5 offset phi (P-ulps) | data tie | sim tie | data parity | sim parity | reading |
  |---|---|---|---|---|---|---|
  | D.S3, 2^-1 (x 0.74-0.98) | 0.50 | 0.426 | 0.025 | 0.032 | 0.188 | Excel D3 bit 2^-54 = 0 |
  | D.S3, 2^0 (x 1.35-1.48) | 0.25 | 0.484 | 0.098 | 0.065 | 0.068 | Excel D3 bit 2^-53 = 0 too |
  | D.S4, 2^-2 / 2^-1 | 0 | 0.21 / 0.43 | 0.19 / 0.41 | small | small | consistent (no information) |
  | D.S5, 2^-3 | 0 | 0.137 | 0.151 | small | small | consistent |
  | N.S2, 2^-2 (x 0.70-0.78) | 0.50 | 0.386 | 0.037 | 0.044 | 0.103 | Excel N2 last bit = 0 |
  | N.S5, 2^-4 | 0.50 | 0.037 | 0.039 | 0.058 | 0.063 | Excel N5 odd (as c5) |

  **So Excel's D3 is a multiple of 4 ulps (c5's ...b69 ends in binary 01; ...b68 fits), N2 is even and N5 is
  odd.** These are bit-level facts about the true coefficients, read without knowing the rest.
- The fixed set (D3 -> ...b68, N2 -> +1 ulp) scores far 0.316 (c5: 0.318) and near 0.7455. The exact score does
  not move, because the line phases (the full stage values) still differ.

## 3. Phase solve: works, but underdetermined so far
- Phase law. Relative to an exact base simulation, the per-run phase of a tie line is
  dphi_r = (S_j^Excel(x_r) - S_j^base(x_r))/(2 ulp S_j) mod 1. That is linear in the integer ulp moves of
  c_j..c_top. Verified on synthetic data: the truth fits to 0.011 cycles rms (`latsolve.py control`, D.S3,
  x < 1). The sign is +1.
- **One line is degenerate.** D.S3 alone (30 runs on [0.74, 0.98]) has many solutions at the same 0.011 rms.
  - Degree-5 integer polynomials can be nearly zero on a short interval.
  - The control recovered a solution 1,600 ulps from the truth.
- **N.S2 plus function rows** (`joint.py`) is still degenerate. LLL plus Babai, and BKZ, do not recover the truth,
  and exact CVP is too slow at 60-165 dimensions.
- **D chain from all tie lines** (`dsolve.py`): D.S3 (47 runs), D.S4 (83) and D.S5 (16), 146 rows over
  [0.70, 1.50], 6 unknowns u3..u8. Control: truth u = (-21972, -22286, 17824, -44, 5401, 6089), which fits all
  rows at 0.037 cycles.
  - Exact CVP on 26 rows overfits (0.020 on its rows, 0.275 on all). At 45 rows it does not finish in 20 min.
  - LLL + BKZ-20 + Babai on 45/90/146 rows gives u = (-4, -22286, 17820, 4, 5027, 13664) at 0.036 cycles:
    - **u4 exact, u5 within 4 ulps**;
    - u6 within about 50, u7 within about 500, u8 within about 1e4 (weak high-power terms);
    - **u3 only modulo 16**: D3 enters S3's phase as the constant u3·ulp(D3)/(2 ulp S3) = u3/8 or u3/16, so
      only u3 mod 16 is visible (-4 ≡ -21972 mod 16).
  - So far-run tie lines pin the mid D coefficients (D4, D5) to the ulp. D3, D6-D8 need other rows: near-1 stage
    sums S_j(1), function preservation, and more lines.
- **Real data, D chain:** the same LLL + BKZ-20 solve (146 rows, bounds 1e5 and 1e6) did not finish in 15
  minutes per bound. That is not run yet; it is the next step, with a smaller row set (about 60 rows) or BKZ-10.
- On real data, D-only moves at bounds 1e3-1e6 give phase rms 0.08-0.15, and exact 0. The real Excel-c5
  difference involves N too (function-preserving), so a real solve must be joint with function rows.

## 4. Lead (5) from part 1 resolved: binade crossings, not function
`../binades.py` lists every binade crossing of c5's intermediates:
- The "Excel minus c5" bias bumps at 0.83-0.85 and 1.13-1.15 sit on the binade crossings of res = t·A, at
  0.8290 and 1.1314.
- The ±1.5-ulp sign flip at 1.08-1.10 sits on a stack of crossings: N.S0/N.P0 at 1.068, D.S0/D.P0 at 1.0797,
  A at 1.0983 and D.S1/D.P1 at 1.107.
- So the apparent function deviation from part 1 is stage-rounding bias at binade crossings. It is no evidence
  about the generator.

## 5. Status and next steps
- Best exact score is unchanged: c5_f99t at 47.25%. The parity-fixed c5 (D3 -> ...b68, N2+1) is equal near 1
  and 0.2 points lower far.
- New facts:
  - the far "unexplained" lines are D.S3 and N.S2 tie lines;
  - Excel's D3 is a multiple of 4 ulps, N2 is even and N5 is odd;
  - the tie-line phase law is validated on synthetic data;
  - a 6-unknown D-chain lattice recovers D4 exactly and D5 within 4 ulps on synthetic data.
- Next:
  1. Run `dsolve.py` on the real data with a smaller lattice.
  2. Add near-1 stage-sum rows and function rows to fix D3 (mod-16 ambiguity) and D6-D8.
  3. Do the same for the N chain with N.S2 plus the N sawtooth lines. Then refit and score exactly.

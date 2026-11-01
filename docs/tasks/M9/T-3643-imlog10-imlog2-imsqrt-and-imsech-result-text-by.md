---
id: T-3643
title: "IMLOG10, IMLOG2, IMSQRT and IMSECH result text by region: sweep both parts across the domain and match Excel's text"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IMLOG10, IMLOG2, IMSQRT and IMSECH print Excel's text (value and formatting) in every region of the complex plane, with any difference found stated as a rule on the region, not a list of points.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports these four producing text that differs from Excel's in some regions. Done work:
T-2117 (hypot modulus of IMLN/IMLOG10/IMLOG2/IMSQRT), T-2530 (IMLOG2's ln 2 division), T-2539 (logs at 1E+200 and
1E-200+1E-200i), T-2572 (IMSQRT range rule, subnormal-square flush), T-2536 (IMSQRT of a negative real), T-2537 and
T-2743/T-2750 (IMSECH last digits). Open owners: T-2894 (subnormal squares whose sum is normal), T-3609 (IMSECH below
|x| = 0.01). No task has swept the four over one grid covering every quadrant, the axes, |z| near 1 (log near 0),
huge and tiny parts, and parts of very different magnitude. Code: functions/engineering/complex/ (log and sqrt
kernels, `engineering::complex::parse` formatter).
Sweep each function over a grid of (x, y) by sign and magnitude (0, 1E-300, 1E-20, 1E-8, 1-1E-15, 1, 1+1E-15, 10,
1E8, 1E20, 1E300) and both orderings of a large and a small part. Classify misses: wrong value (more than 15 digits
off), wrong format (exponent, i placement, sign of zero), wrong type.

## Acceptance criteria
- [x] A probe case (excel-graph) gives Excel's text for the grid for all four functions.
- [x] Every grid cell where the engine is more than a 15-digit unit off, or prints a different shape or type, matches Excel, or is listed in T-2894 or T-3609 when it falls in their bands.
- [x] Cells within 15 digits are left to a follow-up with `defer: phase-1b`.
- [x] Every quirk found is written up in docs/excel-semantics/engineering.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- The trig and hyperbolic #NUM! domain (T-3642).
- Bit-level fits.

## Outcome
- **Probes (excel-graph, 2026-11-01):**
  - probes/t3643-{imlog10,imlog2,imsqrt,imsech}-grid: 4 × 441 cells, each part over 0, ±1E-300, ±1E-20, ±1E-8, ±(1-1E-15), ±1, ±(1+1E-15), ±10, ±1E8, ±1E20 and ±1E300.
  - probes/t3643-imln-points: 10 IMLN cells.
  - probes-numeric/t3643-r4-log-modulus: the fresh sweep, 540 cells.
- **Rules adopted** (docs/excel-semantics/engineering.md, under the IMLN/IMLOG modulus entries):
  - IMLN, IMLOG10 and IMLOG2 take the modulus as the x87 extended root of the double sum of the flushed squares, rounded again to a double (`log_modulus`, via `extended_sqrt`). Before, they used `hypot` (IMLN, IMLOG10) or IMABS's scaled form (IMLOG2). Near |z| = 1 this moves the real part by about 12%: `IMLOG10("0.999999999999999+1E-8i")` is -3.857E-16, not -4.339E-16.
  - The logs are `#NUM!` when ATAN2's quotient `y/x` overflows.
  - IMSQRT takes angle 0 there, so `IMSQRT("1E-300+1E20i")` is `10000000000`.
- **Eval delta:** probes +5 cases with 80 new-coverage known failures, 0 regressions, nothing else changed.
  - Before the fix, the IMLOG2 grid had 12 misses, IMLOG10 14, IMSQRT 18 and IMSECH 64. After it, IMLOG2 has 0, IMLOG10 2, IMSQRT 14 and IMSECH 64.
  - Across the existing suites, IMLN still has 0 failures and IMLOG2 0, IMLOG10 keeps its 1, and IMLOG2 stays 78/78 in probes-numeric im-last-digit-sweep.
- **Cells left:**
  - 60 IMSECH `#NUM!` cells: |y| ≥ 1E20 with |x| ≤ 10, t3643-imsech-grid rows 1-8 and 12-18 of columns J, K, T and U. They belong to T-3642's argument cut (PR #2777).
  - 20 cells one 15-digit unit off, plus 11 such cells of the fresh sweep, go to T-3795 (`defer: phase-1b`).
- **R4 guards** (decision 0188 R4, kind (c), a rounding model chosen from ranked candidates). Scripts are in docs/research/t3643/.
  - **Derivation:** the model was found on 4 points: eng-imln-modulus A1, and t3643-imlog10-grid D5, E4 and the real input A5.
  - **Hold-out (R4.1):** cells with sha1("<case id>!<cell>")[0] < 77, about 30%, with a 5-point margin. The split was stated before the split or the sweep was scored, but after that derivation.
    - The results are 15-digit text, so bit-exact and within-4-ulp are not observable. Counts are the real part within 15 digits, over the IMLN, IMLOG10 and IMLOG2 grid and point cells.

      | | train | hold-out |
      |---|---|---|
      | chosen | 494/494 (100%) | 225/225 (100%) |
      | `hypot` (correctly rounded) | 485/494 (98.2%) | 215/225 (95.6%) |

    - The chosen model beats `hypot` on hold-out, and its hold-out rate equals its train rate.
  - **Fresh sweep (R4.2):** probes-numeric/t3643-r4-log-modulus was generated after the model froze at 8c235dffc (seed 3643).
    - It has 180 points × IMLN, IMLOG10 and IMLOG2: 100 log-uniform over both parts from 1E-150 to 1E150 with both signs, 60 with |z| within 1E-13 of 1, and 20 with one part near ±1.
    - Real parts within 15 digits: chosen 538/540, `hypot` 508/540. `hypot` never wins a point.
    - With the engine as built, 529/540 cells pass. The 11 misses are each one digit in the 15th: 9 in the imaginary part (`atan2/ln 10` and `atan2·K`, not this model) and 2 in IMLOG10's real `log10` (B29, B52, which `hypot` misses too).
  - **Gate A envelope:** all 540 sweep cells are inside, either within 15 digits of Excel or within 1e-9 relative of 50-digit math. Each of the 11 misses is at most 2.6e-15 relative on the differing part (docs/research/t3643/env.py).
  - **No `FITTED` tag:** there is no fitted constant, and nothing below the reproducibility floor was targeted.

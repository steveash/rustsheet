---
id: T-3478
title: "Fits applied past their tagged range: RATE and IRR secant constants, F density SMALL_SHAPE_OFFSET below LARGE_DF_MIN (R4.3)"
milestone: M9
lane: quality
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
Each fit that T-3395 showed applied past the range its `FITTED` tag names either gets measured over the domain it is
used on, under decision 0188 R4's guards, or is bounded to its range with an accurate fallback outside it (R4.3).

## Context
T-3395 (decision 0195, reviewer-excel-fidelity MINOR) tagged these fits. Their tags name ranges narrower than the
inputs they serve:
- crates/rustsheet-engine/src/functions/financial/financial_util.rs `compute_rate`: `OFFSET` and the residual use of
  `TOLERANCE` (fit lab `rate`, 782 cells: nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10, rate -0.75..2) apply to
  every RATE call.
- `irr_secant`: the 1E-3 start offset and 1E-7 residual stop (fit lab `irr`, 263 cells: 2..41 flows, |c| 1E-6..1E+9,
  guess -0.99..100, roots -0.99..99) apply to every IRR call.
- crates/rustsheet-engine/src/functions/statistical/f_density.rs `SMALL_SHAPE_OFFSET`, `STIRLING_ERROR_HALF` and
  `STIRLING_ERROR_ONE` (4,946 cells, df 1..10 and d1 1 up to d2 30) apply to every shape with min(d) below
  `LARGE_DF_MIN` (16), including 10 < min(d) < 16 and max(d) > 10.
For a solver parameter, "accurate math" outside the range is a design question (the secant's trajectory, not a
formula, makes the value), so a fresh sweep over the wider domain may be the right answer rather than a fallback.
T-3364 covers other extrapolated fits (GAMMA.INV, the Stirling error past df 56, the Student-t df bands), not these.

## Acceptance criteria
- [x] For each fit above: a fresh log-uniform sweep over the domain it serves (R4.2) with the Gate A envelope result,
      and either the tag's range widened to that sweep or the fit bounded to its range with the fallback scored.
- [x] Tags and decision 0195's consequences updated; no gated cell regresses.
- [x] `make check` green; reviewers approve.

## Out of scope
- The Student-t cuts (T-3477) and T-3364's fits.

## Outcome
All three fits were measured on fresh sweeps over the domains they serve, outside their fit sets, and each tag now
names that wider range. None is bounded: each still beats the alternatives there, and for the F constants that
includes the accurate fallback. No engine code path or value changed, only doc comments and tags. quality/fitted.json
is unchanged (86 tags, lock 86).

**Sweeps (R4.2).** There are three new probes-numeric cases, filled by excel-graph on 2026-10-31. Each has 300
points drawn by docs/research/t3478/gen_sweeps.py with seed `T-3478/<case>/r4-fresh`. The draws are log-uniform, in
6 strata of 50, each stratum with one axis outside the fit set and the last with every axis outside. Every point
carries a 50-digit mpmath value (dps 60 checked against dps 80) in the case's `.math.tsv`. The models were frozen
long before the sweeps (T-2005, T-3348, T-1533, T-3205, T-2586), and nothing was tuned on them. The tables come from
docs/research/t3478/score_sweep.py. Gate A's envelope is: within 15 digits of Excel, or within 1e-9 relative of
the math, and an error against a number always fails.

*RATE*, t3478-rate-r4-wide. The fit set covers nper 1..1000, |pmt| 1E-6..1E+6, guess -0.5..10 and rates -0.75..2.
The sweep covers rate -0.999..100, nper 1..1E+5, |pv| and |fv| 1E-8..1E+12 and guess -0.999..1000.

| Stratum | Points | Bit-exact | Within 15 digits | Within 1e-9 of math | Inside envelope | Correctly rounded bit-exact |
|---|---|---|---|---|---|---|
| rate -0.999..-0.75 | 50 | 50 | 50 | 28 | 50 | 4 |
| rate 2..100 | 50 | 45 | 45 | 0 | 45 | 0 |
| nper 1000..1E+5 | 50 | 48 | 48 | 7 | 48 | 0 |
| abs pv, fv 1E-8..1E-4 or 1E+8..1E+12 | 50 | 49 | 49 | 16 | 49 | 0 |
| guess -0.999..-0.5 or 10..1000 | 50 | 47 | 47 | 2 | 47 | 0 |
| all axes wide | 50 | 47 | 47 | 16 | 47 | 5 |
| **all** | 300 | 286 | 286 | 69 | 286 | 9 |

Excel returns a number on 100 points, and the engine is bit-exact on all 100. 14 points are outside the envelope,
and on all 14 Excel is `#NUM!` while the engine returns a number:
- 13 converge to the spurious root r = -1 of a type-1, fv = 0 annuity (Calc!A60, A76, A85, A88, A89, A131, A146,
  A171, A213, A237, A243, A256, A265). This is a new quirk, recorded in financial.md "RATE is #NUM! at the spurious
  root r = -1" and filed as T-3547.
- A285 is a noise-floor cell of T-3424's kind (|pv| 8E+10).

Neither cause is the offset or the residual stop. Candidates were run through the fit lab's T-2005 model with
T-3348's hold (docs/research/t3478/rate_variants.py), which equals the engine on 300 of 300. Of Excel's 100 numbers,
the shipped constants keep 100, offset 1e-4 keeps 8, the offset on the other side 14, residual 1e-6 92 and
residual 1e-8 93.

*IRR*, t3478-irr-r4-wide. The fit set covers 2..41 flows, |c| 1E-6..1E+9, guess -0.99..100 and roots -0.99..99. The
sweep uses conventional series and covers 2..150 flows, |c| 1E-12..1E+15, guess -0.9999..1E+8 and roots
-0.9999..1E+6.

| Stratum | Points | Bit-exact | Within 1e-9 of math | Inside envelope | Correctly rounded bit-exact | Excel number, engine error | Excel error, engine number |
|---|---|---|---|---|---|---|---|
| flows 42..150 | 50 | 29 | 21 | 29 | 1 | 21 | 0 |
| abs c 1E-12..1E-6 or 1E+9..1E+15 | 50 | 42 | 32 | 43 | 2 | 2 | 5 |
| root -0.9999..-0.99 | 50 | 47 | 16 | 47 | 13 | 0 | 3 |
| root 99..1E+6 | 50 | 50 | 43 | 50 | 0 | 0 | 0 |
| guess -0.9999..-0.99 or 100..1E+8 | 50 | 32 | 41 | 44 | 3 | 6 | 0 |
| all axes wide | 50 | 32 | 39 | 40 | 1 | 4 | 5 |
| **all** | 300 | 232 | 192 | 253 | 20 | 33 | 14 |

The candidates were run through the fit lab's `irr_secant_d` replica with the 50-step cap and the retry
(docs/research/t3478/irr_variants.py). The replica equals the engine on 284 cells; the other 16 reach the fallback.
Bit-exact counts against Excel: the shipped offset 1e-3 and residual 1e-7 get 236, offset 1e-4 150, offset 1e-2 130,
residual 1e-6 228 and residual 1e-8 221. So the fitted pair stays best past its fit set.

The 47 cells outside the envelope come from other parts of the solver, filed as T-3548:
- 33 return in Excel but are `#NUM!` in the engine: the step cap and the fallback, the family of T-3406, T-3411 and
  T-3347. A 100-step cap would gain 12 of them.
- 13 are an Excel error where the engine returns the root.
- A289 is `#DIV/0!` in Excel. The engine returns a wrong value there, because the absolute 1e-7 residual stop
  passes at once on flows near 1E-11.

*F density*, t3478-f-density-r4-wide. The fit set covers df 1..10, and d1 = 1 with d2 up to 30. The sweep covers
11 <= min(d) <= 15 with max(d) up to 1E+10, and min(d) <= 10 with max(d) 11..1E+10, which is every shape the
constants serve. All 300 points are inside the envelope and within 1e-9 of the math. Against Excel, the engine is
bit-exact on 61 and within 4 ulp on 232. The correctly rounded density, the accurate fallback R4.3 names, gets 15
and 159. The engine leads on every stratum (statistics.md "F density's small-shape constant past its fit set"), so
the constants are kept and the tag covers min(d) 1..15 with max(d) up to 1E+10.

**Check 7.**
- Hold-out (R4.1): nothing new is fitted. This task compares existing fits with their published alternatives on
  points those fits never saw.
- Reproducibility floor (R4.4): no step targets bits below the floor.
- No keep-matched branch was added.
- Tags (R4.3):
  - `OFFSET` and `TOLERANCE` (RATE): 1082 cells, nper 1..1E+5, |pv| and |fv| 1E-8..1E+12, guess -0.999..1000,
    rate -0.999..100.
  - `irr_secant` (IRR): 563 cells, 2..150 flows, |c| 1E-12..1E+15, guess -0.9999..1E+8, roots -0.9999..1E+6.
  - The F constants: 5246 cells, min(d) 1..15 with max(d) up to 1E+10.

RATE's and IRR's swept boxes are finite. The solvers accept any double, so inputs past these boxes (for example
amounts above 1E+15) are still unmeasured.

**Eval.** The 900 cells are in probes-numeric, which is not gated. `make eval-baseline` changes nothing: every
suite's known failures are unchanged, with 0 visibly-off cells in new cases.

**Semantics.**
- financial.md: new entries "RATE and IRR secant constants past their fit sets" and "RATE is #NUM! at the spurious
  root r = -1", and the T-3478 cells added to "IRR on conventional flows where the secant fails".
- statistics.md: new entry "F density's small-shape constant past its fit set".
- Decision 0195's Consequences now record the outcome.

**Follow-ups:** T-3547 (RATE at r = -1) and T-3548 (IRR's 47 wide-sweep cells).

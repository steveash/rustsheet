---
id: T-3549
title: "Gate A slice 3f: reference tables for SUM and AVERAGE, with the near-zero snap ruled on"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: [T-3358]
spec: docs/specs/SPEC-0029-gate-a-accuracy-gate.md
---

## Goal
SUM and AVERAGE, two of the ten M2 functions Gate A counts as missing, each get a committed, oracle-filled `s1` table
with a cited mpmath definition and `make check` running them. The points where Excel's near-zero snap returns 0
against a nonzero exact sum are handled by a recorded rule, not left as unexplained "Excel far from the math"
points.

## Context
- `rustsheet-eval accuracy` on main as of 8ee20308 prints `missing tables 323`. Ten of them are M2 functions (the
  current milestone), and no open task owns SUM or AVERAGE. T-3555 has ROUND, ROUNDUP and ROUNDDOWN. T-3550 (open PR #2656) exempts
  ABS, MIN, MAX, SUMIF and SUMIFS.
- The math is the exact (mpmath) sum of the doubles of the written literals. AVERAGE divides that sum by the count.
- Design question 1, the near-zero snap. docs/excel-semantics/numbers.md, "Near-zero snapping in addition and
  subtraction" (VERIFIED; T-0110, T-1131, T-3048, T-3066, T-3100, T-3145), says every SUM call returns exactly 0 when
  its final addition has opposite-sign addends and |result| < 8 ULP of the running total before the last addend.
  AVERAGE snaps its sum before dividing. So `SUM(0.1,0.2,-0.3)` is 0, while the exact sum is 5.551115123125783e-17:
  Excel's relative error is 1, far above G = 1e-4. This is a deliberate semantic rule, not an inaccurate kernel, so a
  decision 0188 R3 mark (follow the math) would be wrong. Settle one of these with a decision record:
  - (a) The definition includes the snap. It is a deterministic function of the doubles, so the math is 0 there.
  - (b) The definition is the plain exact sum, and the snap points are listed as Excel-over-G points that stay
    Excel-targeted (as T-3544, the owner-ruling task on T-3358's branch in #2648, does for its points).
  - (c) The sampler does not draw cancelling sums, and the probes keep the snap's evidence.
  A uniform sampler draws a cancelling sum only rarely, so the tail stratum should draw near-cancellations on purpose
  (k·2^-52 offsets around the 8-ULP bound, as probes-numeric `snap-addsub/*` does) unless (c) is chosen.
- Design question 2, arity. SUM and AVERAGE are variadic (1 to 255 arguments). `Definition.params` is a fixed tuple,
  so use a `draw` (as RATE does in `defs/annuity.py`) or a small variadic extension of `model.py`. State the arity
  distribution in the docstring: mostly 2 to 10 scalars, some up to 255. Array-literal arguments are optional; if
  used, they must stay under Excel for the web's array-constant limits (text.md mentions refused workbooks).
- Overflow. numbers.md, "Aggregate overflow" (T-0192, VERIFIED, probes/aggregate-overflow), says SUM overflows
  at an intermediate point, as `+` does. `SUM(9E+307,9E+307)` and `SUM(9E+307,9E+307,-9E+307)` are `#NUM!`, but
  `SUM(9E+307,-9E+307,9E+307)` is 9E+307. AVERAGE overflows the same way. In the second case the exact sum is a
  finite 9e307, so the domain test must follow the running total, not the final value. Otherwise those points
  would read as "Excel errors where the math has a number", and the check fails on that. The definition's `domain`
  should mark a point as a domain error when any left-to-right running total of the doubles is beyond the double
  range. Cite the entry, and include both orders in the edge stratum.
- Summation order. Excel adds left to right; the engine's `SumParts` is in
  `crates/rustsheet-engine/src/functions/math_and_trigonometry/sum.rs`. A long sum of mixed magnitudes can leave
  the 15-digit envelope of Excel while staying within ε = 1e-9 of the math. Report how many points pass only
  through the math side.
- Tools: as in T-3555 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite accuracy`,
  `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One definition each for SUM and AVERAGE. Their unit tests cover `SUM(0.1,0.2,-0.3)`,
      `SUM(0.1,0.2,-0.3,0)` = 5.551115123125783e-17 (the trailing-zero case), `AVERAGE(0.1,0.2,-0.3)`, and
      `SUM(9E+307,9E+307,-9E+307)` (a domain error) against `SUM(9E+307,-9E+307,9E+307)` (9E+307). Each test
      checks the value or domain-error outcome the recorded rules give.
- [x] A decision record states the snap rule (a), (b) or (c) and the overflow rule, and the definitions cite it.
- [x] `eval/accuracy/tables/{SUM,AVERAGE}/s1.{case,math,oracle}.json` are generated, filled and committed.
      `rustsheet-eval accuracy` reports `missing tables` 2 lower than before the change. The Outcome names every
      failing point with its owner task (`known-failures.tsv` under T-3379's rules) and gives the count of points
      that pass only through the math side.
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed. `make eval` shows no
      `snap-addsub/*` cell of probes-numeric leaving the R1(c) envelope.
- [x] `make check` green; reviewers approve.

## Out of scope
- SUMIF, SUMIFS, SUMPRODUCT, SUBTOTAL and AGGREGATE (T-3550, and T-3551 filed with it in #2656).
- Changing the engine's snap or summation (file any failing point as its own task).

## Outcome
Done (2026-11-01).
- **Decision 0224 (option (a)):** SUM's and AVERAGE's definitions include the near-zero snap and the running-total
  overflow. Both rules depend only on the doubles. Both read Excel's running total: each addition rounded to 64 bits,
  then to a double (`fitlab.exact.double_round`, the engine's `extended_sum`).
  - The math is 0 where the final addition snaps; elsewhere it is the exact sum, or the exact sum over the count.
  - A running total beyond the double range is a domain error (Excel's `#NUM!`).
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/aggregate.py`):
  - Variadic: `Definition.variadic` and `params_for(n)` repeat the last parameter. `check_domain`, `fit_args`,
    `compute`, `parse_formula` and the sampler's `draw` path use it.
  - Arity: 2 to 10 scalars in typical, tail and edge draws; 50 to 255 in extreme; log-uniform 1 to 255. No array
    literals.
  - The tail stratum draws near-cancelling sums of short decimals, some with a trailing 0. The edge stratum draws
    both overflow orders, one argument, zeros, an exact cancellation and 255 arguments.
  - Tests (`tools/tests/test_accuracy_aggregate.py`) cover `SUM(0.1,0.2,-0.3)` = 0, `AVERAGE(0.1,0.2,-0.3)` = 0,
    `SUM(0.1,0.2,-0.3,0)` (the exact 2.7755575615628914e-17; Excel's 5.551115123125783e-17 is a double rounding
    away), the T-3145 and T-3248 8-ULP bounds with AVERAGE's values, T-2714's subnormal flush, and
    `SUM(9E+307,9E+307,-9E+307)` (a domain error) against `SUM(9E+307,-9E+307,9E+307)` = 9E+307.
- **Tables:** `SUM/s1` and `AVERAGE/s1`, 120 points each, filled from excel-graph.
  - All 240 points are inside the envelope: 0 failing, 0 table defects, no known failures added. `missing tables`
    drops by two (308 to 306).
  - 0 points pass only through the math side.
  - Excel is more than G off the math at 13 SUM and 10 AVERAGE points. All are non-snapped near-cancellations, and
    all pass on Excel's side, where the engine matches Excel to the bit.
  - The tail points bracket the snap bound: snapped at 0, 1 and 7 ULP of the running total, kept from 9 ULP. Excel
    agrees with the definition at every one.
  - Both `#NUM!` overflow points (`SUM(9.09501882444508E+307,9.09501882444508E+307,-9.09501882444508E+307)` and the
    AVERAGE one) are `#NUM!` in Excel.
- **probes-numeric math sidecars:** new for the SUM and AVERAGE cells of `30bc5f8fab02d0a2` (the 216 SUM cells of
  `snap-addsub/*`), `double-rounding-accumulators`, `double-rounding-chained` and `double-rounding-ops`.
  - Every one has Excel within 1e-9 of the math, so no `snap-addsub/*` cell leaves the R1(c) envelope.
  - The probes-numeric R1(c) count is unchanged (39207 counted, 32150 Phase 1b).
  - The `+`/`-` cells of `snap-addsub/*` are not SUM calls and get no sidecar from this definition.

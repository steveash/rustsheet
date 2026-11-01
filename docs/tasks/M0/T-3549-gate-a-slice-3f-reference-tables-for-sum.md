---
id: T-3549
title: "Gate A slice 3f: reference tables for SUM and AVERAGE, with the near-zero snap ruled on"
milestone: M0
lane: harness
status: in-progress
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
- [ ] One definition each for SUM and AVERAGE. Their unit tests cover `SUM(0.1,0.2,-0.3)`,
      `SUM(0.1,0.2,-0.3,0)` = 5.551115123125783e-17 (the trailing-zero case), `AVERAGE(0.1,0.2,-0.3)`, and
      `SUM(9E+307,9E+307,-9E+307)` (a domain error) against `SUM(9E+307,-9E+307,9E+307)` (9E+307). Each test
      checks the value or domain-error outcome the recorded rules give.
- [ ] A decision record states the snap rule (a), (b) or (c) and the overflow rule, and the definitions cite it.
- [ ] `eval/accuracy/tables/{SUM,AVERAGE}/s1.{case,math,oracle}.json` are generated, filled and committed.
      `rustsheet-eval accuracy` reports `missing tables` 2 lower than before the change. The Outcome names every
      failing point with its owner task (`known-failures.tsv` under T-3379's rules) and gives the count of points
      that pass only through the math side.
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed. `make eval` shows no
      `snap-addsub/*` cell of probes-numeric leaving the R1(c) envelope.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SUMIF, SUMIFS, SUMPRODUCT, SUBTOTAL and AGGREGATE (T-3550, and T-3551 filed with it in #2656).
- Changing the engine's snap or summation (file any failing point as its own task).

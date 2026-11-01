---
id: T-3671
title: "Gate A slice 4d: reference tables for ASINH, ACOSH, ATANH and ACOTH"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
ASINH, ACOSH, ATANH and ACOTH each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- numbers.md, "Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once": ACOSH below 1 and ATANH outside (-1, 1) are
  domain errors; ACOTH is defined for |x| > 1.
- Near the domain edges (ACOSH near 1, ATANH and ACOTH near ±1) a naive log formula loses digits; the tail stratum
  should draw there, and any point where Excel is above G off the math is an R3 candidate.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 4. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/inverse_hyperbolic.py`): the true function of the double
  of the written literal.
  - ACOSH takes x >= 1, ATANH (-1, 1) and ACOTH |x| > 1 (numbers.md, T-1202 and T-1569: ±1 is `#NUM!` for ATANH and
    ACOTH).
  - ASINH and ACOSH are `#NUM!` where `x*x` overflows a double (T-2062). The exact cut in [1.3E154, 1.35E154] is
    inferred, so the draws keep out of that band.
  - Excel's plain log formulas (ASINH, ACOSH) and x87 kernels (ATANH, ACOTH) are kernel inaccuracy, not semantic rules.
  - Tests: `tools/tests/test_accuracy_inverse_hyperbolic.py`.
- **Tables:** `s1` for the four functions, 120 points each, filled from excel-graph.
  - All 480 points are inside the envelope: 0 failing, 0 table defects, no known failures added.
  - `missing tables` drops by 4 (277 to 273).
  - Every domain-error point is an Excel `#NUM!` (ASINH 5, ACOSH 7, ATANH 3, ACOTH 8), and every number point is an
    Excel number.
  - ACOSH, ATANH and ACOTH: Excel is within 1.8E-9 relative of the math (ACOSH near 1, the cancellation of its
    formula), 3.6E-13 (ATANH's log branch near 1E-4) and 4.6E-16. `ACOTH(-9.99999999999999E+307)` reads 0 for a subnormal
    true value, inside the absolute floor; the engine keeps the subnormal (-1.000000000000001E-308), so that point
    passes on the math side. Graph reports a kept subnormal as 0 too, so the cell does not tell whether Excel flushes.
  - **R3 candidates (left unmarked, no owner ruling):** 23 ASINH points where Excel is more than G = 1e-4 off the
    math: 22 arguments from 8.5E-285 to 1.6E-17 where Excel's `ln(x + sqrt(x*x + 1))` is 0, and
    `ASINH(6.3701662356807E-14)` (3.9e-4 relative). The engine reproduces Excel's formula there, so each passes on
    Excel's side. The quirk is recorded in numbers.md.
- **probes-numeric math sidecars:** new for m9-hyperbolic, t0917-acoth-switch, t0917-acoth-switch-edge,
  t0917-hyperbolic-switchover, t2062-asinh-acosh-overflow and t2133-asinh-acosh-ln (743 numeric cells plus 15 `#NUM!`
  cells). Excel is within 1e-4 everywhere and within 1e-9 except 7 m9 cells: ASINH at 1E-10 to 3E-8 (A1-A6) and `ACOSH(1.00000001)` (A64),
  as numbers.md records.
  - T-3670 (#2764) adds m9-hyperbolic and t0917-hyperbolic-switchover sidecars for the hyperbolic cells. Whichever of
    the two merges second regenerates those two files with both families.

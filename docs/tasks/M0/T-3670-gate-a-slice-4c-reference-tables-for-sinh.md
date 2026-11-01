---
id: T-3670
title: "Gate A slice 4c: reference tables for SINH, COSH, TANH, COTH, SECH and CSCH"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
SINH, COSH, TANH, COTH, SECH and CSCH each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- The math is mpmath's function at the double of the written literal.
- numbers.md, "SINH / COSH / TANH are exp-based, not libm" and its "COTH, CSCH and SECH" sections (T-0917, T-2169):
  Excel's kernels and their overflow behaviour (COSH(1E+15) is `#NUM!`; SECH past the overflow is 0, not an error).
  The overflow cut of each function is part of the domain.
- COTH(0) and CSCH(0) are poles: check Excel's error before writing the domain.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 6. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/hyperbolic.py`): the true function of the double of the
  written literal, with COTH, SECH and CSCH as reciprocals.
  - SINH and COSH: `#NUM!` once e^|x| overflows a double (numbers.md, T-2169), so the domain is |x| <= ln(DOUBLE_MAX).
    The cut lies in (709.78, 710), unprobed, so the draws keep out of that band.
  - COTH and CSCH: a pole at 0 (gen-m5 `CSCH(0)` = `#DIV/0!`; COTH is 1/TANH).
  - SECH and CSCH have no overflow cut: their 0 past the overflow is an underflow inside Gate A's absolute floor. Their
    draws stop at |x| = 1000, since the math of a huge argument has an exponent hundreds of digits long.
  - Excel's kernels (exp forms, expm1 below 1) are kernel inaccuracy, not a semantic rule, so they are not in the
    definitions.
  - Tests: `tools/tests/test_accuracy_hyperbolic.py`.
- **Tables:** `s1` for the six functions, 120 points each, filled from excel-graph.
  - All 720 points are inside the envelope: 0 failing, 0 table defects, no known failures added.
  - `missing tables` drops by 6 (277 to 271).
  - Every domain-error point is the matching Excel error: SINH 6 and COSH 3 `#NUM!` (710 and 1E+300), COTH 4
    `#DIV/0!` (`COTH(0)`, new evidence, recorded in numbers.md).
  - Excel is within 3.3E-16 relative of the math at every normal true value. It is more than G = 1e-4 off at 19 SECH
    and CSCH points, all true values below the smallest normal double (|x| >= 709.78) that Excel shows as 0, inside
    the absolute floor. No R3 candidate.
- **probes-numeric math sidecars:** new for m9-hyperbolic, t0917-hyperbolic-switchover, t2178-hyperbolic-unit-sweep,
  t3395-hyperbolic-form-switch (and -2) and power-ulp-sources (3 TANH cells): 3603 numeric cells plus 3 `#NUM!` cells
  (SINH(710), SINH(711), SINH(-710)), which match Excel. Excel is within 1e-9 of the math at every numeric one except `CSCH(710)`, which is 0 for a subnormal true value.
- **Follow-up:** T-3748 (one shared home for `scalars` and the zero-pole helper, which trig.py and hyperbolic.py both define).

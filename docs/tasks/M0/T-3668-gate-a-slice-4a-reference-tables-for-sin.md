---
id: T-3668
title: "Gate A slice 4a: reference tables for SIN, COS, TAN, COT, SEC and CSC"
milestone: M0
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
SIN, COS, TAN, COT, SEC and CSC each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- The math is mpmath's function at the double of the written literal.
- numbers.md, "Trig of a huge angle": each is `#NUM!` from |x| = 2^27 on (fractions included), a domain error for the
  definition. COT and CSC at 0, and SEC and TAN at no double, are poles: check which errors Excel gives (`#DIV/0!` for
  COT(0) and CSC(0)) before writing the domain.
- Excel's argument reduction is documented: numbers.md "Trig of a huge angle" (reduction by the x87 64-bit π/2) and
  "SIN/COS/TAN/SEC/CSC/COT near multiples of PI(): oracle sweep (T-1008)" (probes-numeric/m9-trig-reduction, 666
  cells). The definition reads that rule; only a point the documented rule does not explain is an R3 candidate.
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
Done (2026-11-01).
- **Definitions** (`tools/src/rustsheet_tools/accuracy/defs/trig.py`): SIN, COS, TAN, COT, SEC and CSC are the true
  functions of the double of the written literal (SEC, CSC and COT as reciprocals).
  - Domain: the open interval (-2^27, 2^27) (numbers.md "Trig of a huge angle": `#NUM!` from |x| = 2^27).
  - COT and CSC at 0 are domain errors: gen-m5 (excel-graph) has `COT(0)` and `CSC(0)` at `#DIV/0!` and `SEC(0)` at 1.
  - Excel's π₆₄ argument reduction (the x87 64-bit π) is the inaccuracy of its constant, not a semantic rule, so it is
    not part of the definition. Such points pass on Excel's side, where the engine reproduces the model bit for bit
    (T-1563).
  - One shared draw: typical in (-10, 10); a tail near k·π/2 (k up to 1E+6, offsets down to 1E-12) where the reduction
    shows; edges just inside and past the cut, 0 and tiny arguments; extreme and log-uniform magnitudes up to the cut.
  - Tests: `tools/tests/test_accuracy_trig.py`.
- **Tables:** `s1` for each of the six functions, 120 points each, filled from excel-graph.
  - All 720 points are inside the envelope: 0 failing, 0 table defects, no known failures added.
  - `missing tables` drops by 6 (277 to 271).
  - Excel is more than G = 1e-4 off the math at one point, a COT near a multiple of π/2. The π₆₄ model reproduces
    Excel's value there, so it is explained by the documented rule and is no R3 candidate. A 15-digit literal rarely sits
    close enough to a multiple of π/2 for the reduction shift to reach G.
- **probes-numeric math sidecars:** new for `t2147-tan-double-rounding`, `t2664-cot-double-rounding` and
  `trig-odd-even-sumxmy2-sweep` (735 cells). The probes-numeric R1(c) count is unchanged (34401 counted, 37228
  Phase 1b): the engine already matches Excel on these cells.

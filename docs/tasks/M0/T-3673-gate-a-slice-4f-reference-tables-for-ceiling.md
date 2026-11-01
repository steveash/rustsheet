---
id: T-3673
title: "Gate A slice 4f: reference tables for CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH and FLOOR.PRECISE"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3555]
---

## Goal
CEILING, CEILING.MATH, CEILING.PRECISE, FLOOR, FLOOR.MATH and FLOOR.PRECISE each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- Reuse the decimal-math rule of T-3555 (the written 15-digit decimal rounded exactly), so the rounding family shares
  one reading of "the math" of a rounding function.
- The sign rules of CEILING and FLOOR with a negative significance (Excel's legacy `#NUM!` for opposite signs) and the
  mode argument of the .MATH forms are domain rules to cite from the semantics entries (search numbers.md for
  CEILING/FLOOR before writing the domain).
- A huge number over a tiny significance may overflow or lose the quotient's digits; such points are R3 candidates.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [ ] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 6. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

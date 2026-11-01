---
id: T-3668
title: "Gate A slice 4a: reference tables for SIN, COS, TAN, COT, SEC and CSC"
milestone: M0
lane: harness
status: todo
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
- Excel's argument reduction near multiples of π/2 may leave Excel far from the math (relative error above G = 1e-4
  where the result is near 0). Such points are R3 candidates that need an owner ruling, not marks.
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

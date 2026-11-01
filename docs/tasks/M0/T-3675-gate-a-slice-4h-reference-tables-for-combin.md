---
id: T-3675
title: "Gate A slice 4h: reference tables for COMBIN, COMBINA, FACT, FACTDOUBLE, MULTINOMIAL, QUOTIENT and MOD"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
COMBIN, COMBINA, FACT, FACTDOUBLE, MULTINOMIAL, QUOTIENT and MOD each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- numbers.md: "COMBIN is inexact", "COMBINA is COMBIN(n+k-1, k)", "FACT and FACTDOUBLE of a huge argument",
  "QUOTIENT, FACTDOUBLE and SERIESSUM argument modes", "MOD is an exact remainder" and "MOD is #NUM! for a huge
  quotient". The math is the exact combinatorial value (or remainder); Excel's product order makes large results
  inexact, which Gate A's envelope judges.
- Arguments truncate to integers (FACT(5.9) is 120): the definition reads the truncated value.
- MULTINOMIAL is variadic: use `Definition.variadic` (T-3549).
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [ ] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 7. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

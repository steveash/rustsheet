---
id: T-3674
title: "Gate A slice 4g: reference tables for ECMA.CEILING, ISO.CEILING, MROUND, EVEN, ODD, INT and TRUNC"
milestone: M0
lane: harness
status: in-progress
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3555, T-3673]
---

## Goal
ECMA.CEILING, ISO.CEILING, MROUND, EVEN, ODD, INT and TRUNC each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- Reuse the decimal-math rule of T-3555.
- numbers.md: "INT cuts to 15 significant digits first", "ODD above 2^53 adds one in binary", "MROUND rounds the
  quotient up within 5E-15 of a half, and multiplies in binary" and "TRUNC is ROUNDDOWN". Each is a rule the definition
  must read (or an R3 candidate if the entry shows Excel's arithmetic, not a rule).
- ECMA.CEILING is not in Microsoft's list, but Excel evaluates it as two-argument CEILING and refuses one argument
  (probes/ecma-ceiling, T-1301; eval/catalog/excel-functions.tsv).
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

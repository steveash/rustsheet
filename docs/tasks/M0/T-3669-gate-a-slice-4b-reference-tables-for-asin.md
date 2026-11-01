---
id: T-3669
title: "Gate A slice 4b: reference tables for ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES and RADIANS"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
ASIN, ACOS, ATAN, ATAN2, ACOT, DEGREES and RADIANS each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- numbers.md, "ACOS is PI()/2 - ASIN(x)": Excel's ACOS is computed through ASIN, so near x = 1 it may be far from the math
  (an R3 candidate needing an owner ruling, or a definition that reads Excel's formula if the entry shows it is a
  semantic rule).
- numbers.md, "Out-of-domain ACOS/ASIN/ACOSH/ATANH are `#NUM!` at once": the domain is [-1, 1].
- ATAN2 takes (x, y) in Excel's order, and ATAN2(0,0) is `#DIV/0!`.
- numbers.md, "RADIANS, SQRTPI and MUNIT": RADIANS is `x * (PI/180)` to the bit; DEGREES is not yet in an entry.
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

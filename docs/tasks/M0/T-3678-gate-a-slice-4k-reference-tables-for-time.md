---
id: T-3678
title: "Gate A slice 4k: reference tables for TIME, HOUR, MINUTE, SECOND and YEARFRAC"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3551]
---

## Goal
TIME, HOUR, MINUTE, SECOND and YEARFRAC each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- dates.md, "TIME divides by 86400 first, then drops whole days" and "TIME: an argument past 32767 is `#NUM!`; a
  near-integer hour counts as that integer (T-2267)": TIME's math reads Excel's rule.
- dates.md, "HOUR, MINUTE and SECOND round to the nearest second" and "HOUR, MINUTE and SECOND past 9999-12-31": the
  integer result follows Excel's rounding of the fractional day, so the definition reads that rule (they are not
  `exact`, decision 0228).
- dates.md, "YEARFRAC accepts serial 0", and the basis argument (0-4): YEARFRAC's math per basis is the day-count
  convention; cite the semantics entries for each basis before writing it.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [ ] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 5. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

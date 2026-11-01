---
id: T-3676
title: "Gate A slice 4i: reference tables for PRODUCT, SUMSQ, SERIESSUM and PERCENTOF"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3549, T-3586]
---

## Goal
PRODUCT, SUMSQ, SERIESSUM and PERCENTOF each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- PRODUCT and SUMSQ are variadic: use `Definition.variadic` (T-3549). numbers.md "Overflow in `+ - * /` is #NUM! where it happens" (its aggregate-overflow
  paragraph, T-0192): PRODUCT and SUMSQ overflow at an intermediate point like SUM (`PRODUCT(1E+200,1E+200)` stays `#NUM!` with a later factor of 0), so
  the domain follows the running product or sum, as T-3549's SUM does (decision 0224).
- SERIESSUM(x, n, m, coefficients) takes an array literal of coefficients (T-3586's array kind); numbers.md "QUOTIENT,
  FACTDOUBLE and SERIESSUM argument modes".
- PERCENTOF(subset, all) is SUM(subset)/SUM(all) over two arrays; probes/percentof-snap-overflow-union (T-0416) holds
  Excel's evidence on whether its sums snap and overflow: read it before writing the definition.
- Tools: as in T-3549 and T-3586 (`rustsheet-accuracy generate`, `rustsheet-oracle fill -b excel-graph --suite
  accuracy`, `rustsheet-eval accuracy`, then `rustsheet-accuracy math --suite probes-numeric`).

## Acceptance criteria
- [ ] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [ ] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 4. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [ ] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

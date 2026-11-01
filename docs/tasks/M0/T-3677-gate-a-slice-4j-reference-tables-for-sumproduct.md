---
id: T-3677
title: "Gate A slice 4j: reference tables for SUMPRODUCT, SUMX2MY2, SUMX2PY2 and SUMXMY2"
milestone: M0
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3551, T-3549, T-3586]
---

## Goal
SUMPRODUCT, SUMX2MY2, SUMX2PY2 and SUMXMY2 each get a committed, oracle-filled `s1` Gate A table with a cited mpmath definition, and
`make check` runs them (decision 0228 classified them as needing a table).

## Context
- Filed by T-3551 (decision 0228): these in-scope numeric functions have a floating-point kernel, so neither `exact`
  nor `composite` fits and Gate A (decision 0188 R1(a), SPEC-0029) needs their tables.
- Each takes two (SUMPRODUCT: two or more) equal-length array literals (T-3586's array kind). The math is the exact sum
  of the products (or of x²-y², x²+y², (x-y)²) of the doubles.
- SUMPRODUCT never snaps near zero (numbers.md, near-zero snap, "Summing functions": T-3048,
  probes/t3048-snap-conditional-sums), so its math is the plain exact sum. Whether the SUMX family snaps, and whether
  each overflows at a running total, is to be probed before writing the domain.
- Unequal lengths are `#N/A` (SUMX2MY2 family) or `#VALUE!` (SUMPRODUCT): domain errors, an edge point each.
- numbers.md mentions trig-odd-even-sumxmy2 probes (probes-numeric/trig-odd-even-sumxmy2-sweep): reuse their evidence.
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

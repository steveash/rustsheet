---
id: T-3674
title: "Gate A slice 4g: reference tables for ECMA.CEILING, ISO.CEILING, MROUND, EVEN, ODD, INT and TRUNC"
milestone: M0
lane: harness
status: done
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
- [x] One Gate A definition per function in `tools/src/rustsheet_tools/accuracy/defs/`, each citing its sources, with unit
      tests against published or semantics-entry values.
- [x] `eval/accuracy/tables/<FUNCTION>/s1.{case,math,oracle}.json` for each function, generated, filled (excel-graph) and
      committed; `rustsheet-eval accuracy` reports `missing tables` lower by 7. The Outcome names every failing point
      with its owner task (`known-failures.tsv`, T-3379's rules) and any R3 candidate (decision 0188 R3 needs an owner
      ruling).
- [x] The probes-numeric math sidecars are regenerated with the new definitions and committed.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the engine's kernels: each failing point gets its own task (`known-failures.tsv`).

## Outcome
- Definitions (decision 0246): `defs/integer.py` (INT, EVEN, ODD, MROUND, new), TRUNC in `defs/rounding.py` (ROUNDDOWN's
  rule and draws), ECMA.CEILING and ISO.CEILING in `defs/ceiling_floor.py` (CEILING's and CEILING.PRECISE's). INT reads its
  15-digit cut, MROUND takes its step count from Excel's double quotient with the 5E-15 tolerance (an exact-ratio
  reading would put decimal halves a whole step from Excel), EVEN and ODD are exact at the double. Unit tests:
  `tools/tests/test_accuracy_integer.py`, against the values numbers.md cites.
- Tables: `s1` for each of the seven (120 points each), filled from excel-graph 2026-11-02; `rustsheet-accuracy verify`
  0 differences. `rustsheet-eval accuracy`: 5616 points in 35 tables, 0 failing, 0 table defects; missing tables
  265 → 258 (−7). No failing point, so no `known-failures.tsv` line.
- Excel against the math: the widest gap is 8.4e-15 relative (ECMA.CEILING/s1 A113, the binary quotient of decision
  0244's last consequence), so there is no R3 candidate. All 18 math domain errors (ECMA.CEILING 8, ISO.CEILING 1,
  MROUND 9) are errors in Excel.
- probes-numeric math sidecars regenerated: `trig-odd-even-sumxmy2-sweep` gains math for its 132 ODD and EVEN cells; no
  other committed sidecar changed.
- Eval delta: none (no engine change).

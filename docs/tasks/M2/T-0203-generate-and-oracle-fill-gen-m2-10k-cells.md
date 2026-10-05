---
id: T-0203
title: "Generate and oracle-fill gen-m2 (≥ 10k cells)"
milestone: M2
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: [T-0202]
---

## Goal
Oracle-backed coverage for all M2 functions.

## Context
-

## Acceptance criteria
- [x] gen-m2 ≥ 10,000 oracle cells; gate = true; baseline recorded; gap tasks filed.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
30 cases (seed 1, 400 formulas each), 12,000 scored cells (12,108 oracle cells), 85.62% bit-exact; `gate = true`; baseline 1,726 known failures (new coverage). The old catalog let `IF` take one argument, which Excel rejects for the whole workbook: catalog fixed, `tools/tests/test_catalog_arity.py` added, rule in excel-semantics/syntax.md. T-0208 (oracle `#SPILL!` decoding) fixed here too. Gap-triager: 15 tasks (T-0208..T-0222), T-0125/T-0106/T-0107 extended; 13 quirks recorded (criteria, lookup, references, numbers, coercion, dates).

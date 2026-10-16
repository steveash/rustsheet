---
id: T-1607
title: "Suite gen-m8b: tables and structured references, oracle-filled"
milestone: M8
lane: harness
status: in-progress
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A committed, seeded suite `gen-m8b` with excel-graph values scores Excel tables and structured
references.

## Context
Owner request (2026-10-15): finish Phase 1's test suite before the Phase 2 re-architecture
(decision 0027). Building test infrastructure is allowed during Phase 1, and this is one of the
suites the milestone exit criteria name that does not exist yet.
- spec2xlsx can already write tables (`"tables": [{"name": "T1", "ref": "A1:B6"}]`, T-1036), with
  a literal-text header row.
- Structured references need a generator policy that knows the table layout. Cover:
  - `Table1[Col]`, `Table1[@Col]`, `Table1[[#Headers],[Col]]`;
  - `#All`, `#Data`, `#Totals`, `#This Row`, and column ranges `Table1[[A]:[C]]`;
  - escaped column names (`'`, `[`, `]`, `#`), references from outside and inside the table,
    and totals-row formulas (SUBTOTAL 109 etc.).
- Check that excel-graph keeps structured-reference formulas on upload. If it rewrites them,
  record what it does in docs/excel-semantics/.

## Acceptance criteria
- [ ] A generator policy produces the cases above, with tables of 1–5 columns, 0–20 data rows,
  with and without a totals row.
- [ ] `eval/suites/gen-m8b` is committed with ≥ 3,000 oracle-filled, canary-verified cells,
  `gate = false`, `milestone = ["M8"]`. Every failing group has a filed or cited task.
- [ ] Quirks recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (file them). Table resizing and other edits.

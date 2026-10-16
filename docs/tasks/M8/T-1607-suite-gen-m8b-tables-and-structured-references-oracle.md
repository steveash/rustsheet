---
id: T-1607
title: "Suite gen-m8b: tables and structured references, oracle-filled"
milestone: M8
lane: harness
status: todo
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

## Handoff
- Done (no code yet): checked excel-graph against spec2xlsx tables with structured references, using throwaway probes on table `Sales` (A1:D4, columns A B C D, 3 data rows).
  - Keeps and computes these on upload: `=SUM(Sales[A])` 6; `=Sales[[#Headers],[B]]` "B"; `=ROWS(Sales[#All])` 4; `=SUM(Sales[[A]:[B]])` 66; `=COUNTA(Sales[#Data])` 12; `=FORMULATEXT(..)` of `=SUM(Sales[A])` gives "=SUM(Sales[A])".
  - Name tables like `Sales`, not `T1`: XlsxWriter rejects a table name that looks like a cell address.
- Blockers in spec2xlsx (tools), to fix before the generator:
  - `=ROWS(Sales)` (a bare table name) makes excel-graph refuse the whole file (createSession 501 "FileCorruptTryRepair / unsupportedWorkbook"). spec2xlsx writes it as a dynamic-array formula (`<f t="array" ref=..>`, `cm="1"`). Find out why its array classifier picks this formula.
  - `Sales[@A]*2` inside the table is also written as an array formula, and `@` is not translated to the file form `Sales[[#This Row],[A]]`. excel-graph returns `#REF!`. That is a writer artifact, not Excel behavior.
  - A case that also had an escaped column (header `C#`, `=SUM(Sales['C#])`) and `=COLUMNS(Sales[#Headers])` still failed upload. Bisect those two next (one formula per case; `rustsheet-oracle fill` takes about a minute).
- Next: fix the spec2xlsx structured-reference writing above, with tests in tools. Then write the generator policy (tools/src/rustsheet_tools/gen/, as names_3d.py does for gen-m8a) and the suite.
- Pause reason: session time budget (opus worker, 2026-10-16). No attempt failed.

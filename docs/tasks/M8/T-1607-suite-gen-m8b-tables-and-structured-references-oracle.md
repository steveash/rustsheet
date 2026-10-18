---
id: T-1607
title: "Suite gen-m8b: tables and structured references, oracle-filled"
milestone: M8
lane: harness
status: done
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
- [x] A generator policy produces the cases above, with tables of 1–5 columns, 0–20 data rows,
  with and without a totals row.
- [x] `eval/suites/gen-m8b` is committed with ≥ 3,000 oracle-filled, canary-verified cells,
  `gate = false`, `milestone = ["M8"]`. Every failing group has a filed or cited task.
- [x] Quirks recorded in docs/excel-semantics/.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (file them). Table resizing and other edits.

## Outcome
`rustsheet-gen --policy tables` (tools/src/rustsheet_tools/gen/tables.py) writes 9 cases, one per
category: column, specials, column-range, this-row, in-table, escaped, totals, lookups and
names-case. Each holds four seeded tables (`Sales`, `Stock`, `Tiny`, `Wide`; 1-5 columns, 0-20 data
rows, with and without totals rows, plain and escaped headers) on `Data` and probes on `Calc`, on
`Data` beside the tables (this-row) or in formula columns inside two tables (in-table).
eval/suites/gen-m8b holds 3,368 probes, all filled by excel-graph (none pending). The engine passes
2,096 (62.2%); column passes in full. Failing groups, each filed:
- the `@` this-row shorthand (in-table 23/160, most of this-row 59/400): T-1672;
- case-insensitive names and several item specifiers (names-case 32/400, part of specials): T-1673;
- a bare table name and `Table[]` (`#NAME?`; lookups 371/400, specials 268/400): T-1674;
- `#This Row` outside the table's rows and reversed column ranges (an unsigned wrap): T-1675;
- totals rows, which the facade cannot add (totals 187/408, `#All` counts): T-1671.

spec2xlsx now stores structured references in their file form (a bare name as `Table[]`, `@` as
`#This Row`) and writes totals rows (`"totals_row": true`) with the metadata Excel needs; excel-graph
refused the handoff's files for exactly these reasons. The engine adapter registers a totals-row
table without that row until T-1671. Quirks: docs/excel-semantics/references.md "Structured
references: this row, totals and ranges" and "... names and item specifiers", xlsx-storage.md
"Structured references in the file".

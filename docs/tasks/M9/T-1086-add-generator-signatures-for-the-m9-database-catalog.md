---
id: T-1086
title: "Add generator signatures for the M9 database catalog functions"
milestone: M9
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M9` can generate formulas for the database functions (12 in scope): each in-scope catalog
entry with milestone M9 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. One of the per-category tasks split from the M9 signature work; categories here: database.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M9` and category database (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m9` sample (`rustsheet-gen --milestone M9`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
- **Signatures.** All 12 in-scope M9 database entries now carry signatures in eval/catalog/functions.yaml,
  and the new test_generator_emits_every_m9_database_function checks that the generator emits all 12.
- **Modes.** They come from the new probe probes/db-argument-modes (121 cells, excel-graph, 2026-10-11;
  statistics.md "Database function argument modes").
  - `database` and `criteria` are `range`. A database given as an array literal makes Excel refuse the
    workbook.
  - `field` is `value` with a soft 1-4 index domain. It lifts over array literals but not over ranges.
- **Seeded sample.** `rustsheet-gen --milestone M9 --count 20 --seed 17` was run from a scratch catalog
  holding only the database signatures, and was not committed. It called all 12 functions. excel-graph
  accepted all 20 workbooks (0 failures), and the engine passed 673/800 cells.
- **Eval delta.** probes gained 1 case: 121 cells, 83 pass, 38 new known failures (T-1389). Regressions: 0.
- **Follow-ups.**
  - T-1389: lifting, and the empty field in DCOUNT/DCOUNTA.
  - T-1390: header matching, criteria and error choice on generated data.

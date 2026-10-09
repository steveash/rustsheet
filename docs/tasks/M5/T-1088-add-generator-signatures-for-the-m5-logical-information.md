---
id: T-1088
title: "Add generator signatures for the M5 logical, information and web catalog functions"
milestone: M5
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M5` can generate formulas for the math, logical, text and information functions (134 in scope, decision 0027): each in-scope catalog
entry with milestone M5 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. One of the per-category tasks split from the M5 signature work; categories here: logical, information and web.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M5` and category logical, information and web (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m5` sample (`rustsheet-gen --milestone M5`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 26 `scope: in` M5 logical, information and web entries (INFO and SHEETS are `oracle: none`) now carry signatures
in eval/catalog/functions.yaml. test_catalog_arity.py checks them against excel-functions.tsv, and test_generator.py
checks that the generator emits all 26 over 40 seeds. Modes come from evidence: the existing IS*, ISEVEN/ISODD, IFS/SWITCH entries and new probes
(probes/info-logic-argument-modes, -2, info-modes-sheet-text, info-modes-isref-value-op; excel-graph 2026-10-09).
XOR is `aggregate`, TYPE, ISREF and SHEET take a range whole (`array`), ISFORMULA and CELL's reference are `range`
(a literal makes excel-graph refuse the workbook), and the rest are `value`. The generator gained three catalog
keys: `domain.enum` (CELL's info_type), `cell: true` (pass one cell to a `range` param) and `always: true` (never
omit CELL's reference). A seeded sample (`rustsheet-gen --milestone M5 --count 25 --seed 11`, not committed) was all
accepted by excel-graph (0 failures). The engine passed 922 of 960 cells; one case timed out on
`MULTINOMIAL` of about 1e30 (T-1197; first filed as T-1217, a duplicate, since removed). Quirks: information.md "Information function argument modes",
logic.md "XOR argument modes", web.md "ENCODEURL is #VALUE! in Excel for the web". Eval delta: 4 new probe cases,
56 cells, 39 pass, 17 new known failures (T-1218, T-1219; `CELL("address")` across sheets stays unimplemented).
Follow-ups: T-1197 (existing), T-1218, T-1219.

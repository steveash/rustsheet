---
id: T-1090
title: "Add generator signatures for the M6 date and time catalog functions"
milestone: M6
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M6` can generate formulas for the date & time functions (20 in scope; lookup & reference split to T-1238): each in-scope catalog
date entry with milestone M6 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. Size M: split by category
if the signatures do not fit one PR. Split on 2026-10-09: this task signs the 20 date and time functions; the 19 lookup
and reference functions are T-1238.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M6` and category date (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m6` sample (`rustsheet-gen --milestone M6`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 20 `scope: in` M6 date and time entries now carry signatures in eval/catalog/functions.yaml (date parameters
drawn as serials 1-60000 like YEAR's, DATEDIF units and DATEVALUE/TIMEVALUE texts as enums, `holidays` as `array`,
everything else `value`); test_catalog_arity.py checks them against excel-functions.tsv and test_generator.py checks
the generator emits all 20 over 40 seeds (the three per-category emit tests now share an `emitted` helper). Modes come
from the new probe probes/date-argument-modes (168 cells, excel-graph 2026-10-09): twelve functions lift over ranges
and arrays, the eight Analysis ToolPak-era ones lift over arrays only and are `#VALUE!` for a range (dates.md "Date
function argument modes"). A seeded sample (`rustsheet-gen --milestone M6 --count 25 --seed 13`, not committed) was all
accepted by excel-graph (0 failures) and the engine passed 957/1000 cells. Eval delta: probes +1 case, 168 cells, 73
pass, 95 new known failures (T-1235, T-1236, T-1237). Follow-ups: T-1235, T-1236, T-1237, T-1238 (lookup signatures),
T-1239 (commit gen-m6 and triage its gaps); also T-1233 (FACT/FACTDOUBLE unbounded loop, found in T-1217's duplicate run).

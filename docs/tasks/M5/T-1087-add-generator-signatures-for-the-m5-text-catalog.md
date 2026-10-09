---
id: T-1087
title: "Add generator signatures for the M5 text catalog functions"
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
another evidence source before a generated suite can score them. One of the per-category tasks split from the M5 signature work; categories here: text.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M5` and category text (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m5` sample (`rustsheet-gen --milestone M5`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 33 `scope: in` M5 text entries (none is `oracle: none`) now carry signatures in
eval/catalog/functions.yaml; test_catalog_arity.py checks them against excel-functions.tsv and
test_generator.py checks the generator emits all 33 over 40 seeds. Modes come from new probes
(probes/text-argument-modes and text-modes-*, excel-graph 2026-10-09): everything lifts (`value`)
except TEXTJOIN's text and ARRAYTOTEXT's array (`array`) and PHONETIC (`range`; a literal argument
makes excel-graph refuse the workbook). A seeded sample (`rustsheet-gen --milestone M5 --count 25
--seed 7`, not committed) first had one workbook refused: MUNIT with a data-derived dimension. The
generator now honors `domain.hard: true` (MUNIT's dimension) and the regenerated 25 workbooks were
all accepted (0 failures); the engine passed 935/960 cells. Quirks: text.md "Text function argument
modes". Eval delta: +63 probe cells, 51 pass, 12 new known failures (T-1212, T-1213, T-1214).
Follow-ups: T-1212, T-1213, T-1214.

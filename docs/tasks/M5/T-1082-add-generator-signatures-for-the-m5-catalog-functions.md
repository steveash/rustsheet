---
id: T-1082
title: "Add generator signatures for the M5 math catalog functions"
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
another evidence source before a generated suite can score them. One of the per-category tasks split from the M5 signature work; categories here: math.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M5` and category math (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m5` sample (`rustsheet-gen --milestone M5`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 73 `scope: in` M5 math entries (none is `oracle: none`) now carry full signatures in
eval/catalog/functions.yaml; tools/tests/test_catalog_arity.py checks each against
excel-functions.tsv. Argument modes come from a new probe, probes/math-argument-modes (56 cells,
excel-graph 2026-10-08): GCD, LCM and MULTINOMIAL error on text/boolean range cells, SUMPRODUCT
reads every non-number as 0 (even typed `"2"`), SERIESSUM's coefficients reject numeric text. That
needed one new mode, `array` (ranges taken whole; the generator passes a range half the time), and
a `volatile: true` flag: RAND and RANDBETWEEN keep signatures but the generator never calls them.
Priors are hand-set (no corpus share exists for most). Acceptance run: `rustsheet-gen --milestone M5
--count 10 --seed 1` + `rustsheet-oracle fill -b excel-graph`: 10 workbooks accepted, 0 failures,
61 of the 71 generatable M5 math functions in this small sample (the unit test sees all 71 over 40 seeds); the engine passed 350/360 cells (not committed; T-1192).
Eval delta: +56 probe cells, 47 pass; 9 new known failures (MULTINOMIAL T-1190, SERIESSUM T-1191).
Quirks recorded: coercion.md "GCD / LCM / MULTINOMIAL range cells", numbers.md "MULTINOMIAL is inexact".
Follow-ups: T-1190, T-1191, T-1192.

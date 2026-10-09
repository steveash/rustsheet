---
id: T-1080
title: "Add generator signatures for the M9 statistical catalog functions"
milestone: M9
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M9` can generate formulas for the statistical functions (142 in scope): each in-scope catalog
entry with milestone M9 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. One of the per-category tasks split from the M9 signature work; categories here: statistical.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M9` and category statistical (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m9` sample (`rustsheet-gen --milestone M9`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 142 in-scope M9 statistical entries now carry signatures in eval/catalog/functions.yaml (none is `oracle: none`);
test_catalog_arity checks each required count against excel-functions.tsv, and the new
test_generator_emits_every_m9_statistical_function checks that the generator emits all 142. Modes come from the new probe
probes/stat-argument-modes (573 cells, excel-graph 2026-10-09; statistics.md "Statistical function argument modes"):
every scalar argument lifts over arrays and ranges (`value`), the data arguments are `aggregate` (STDEV, VAR, MEDIAN, ...),
`aggregate_a` (the A-variants) or `array` (the paired and k-th statistics, LINEST family, FREQUENCY), and RANK's ref,
COUNTBLANK and the *IFS ranges are `range`. Integer arguments (degrees of freedom, trials, k, quart, tails, type,
significance, ETS options) carry soft `int` domains. A seeded sample (`rustsheet-gen --milestone M9 --count 20 --seed 17`,
not committed; 232 calls to 107 of the 142 functions) was all accepted by excel-graph (0 failures), and the engine passed
746/800 cells with no abort.
Eval delta: probes +1 case, 573 cells, 170 pass, 403 new known failures (T-1290 lifting 296 cells; T-1291, T-1292,
T-1293; the rest are ulp-level distribution values already described in statistics.md). Regressions 0.
Follow-ups: T-1289 (commit gen-m9), T-1290, T-1291, T-1292, T-1293.

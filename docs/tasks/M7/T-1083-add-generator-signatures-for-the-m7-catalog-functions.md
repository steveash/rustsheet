---
id: T-1083
title: "Add generator signatures for the M7 catalog functions"
milestone: M7
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M7` can generate formulas for the dynamic-array and LAMBDA functions (32 in scope): each in-scope catalog
entry with milestone M7 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. Size M: split by category
if the signatures do not fit one PR.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M7` (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m7` sample (`rustsheet-gen --milestone M7`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 32 in-scope M7 entries now carry signatures in eval/catalog/functions.yaml (RANDARRAY's is `volatile`, so it is
never generated); the required-argument counts match excel-functions.tsv (test_catalog_arity). The generator gained
`mode: lambda` (a LAMBDA whose body uses its parameters, or an eta-reduced `_xleta.SUM`; `arity: repeat` for MAP),
`mode: name` with `body`/`invoke` (LET variables in scope after their value, LAMBDA parameters in the body, a LAMBDA
called at once), `full` and `mask` (same-height arrays for GROUPBY, SORTBY and FILTER's include), `vector` on `array`
params (WRAPROWS/WRAPCOLS) and `spelled` (`_xlfn.GROUPBY`/`_xlfn.PIVOTBY`, which the case builder leaves bare).
Older milestones generate exactly as before (no new RNG draws without a bound name). Modes come from the new probe
probes/m7-argument-modes (53 questions, excel-graph 2026-10-09; arrays.md "Dynamic-array and LAMBDA function argument
modes"): scalar arguments lift, delimiters, SORT's sort_index/sort_order and FILTER's if_empty are taken whole, and
eta names work wherever a LAMBDA does. A seeded sample (`rustsheet-gen --milestone M7 --count 25 --seed 17`, not
committed) was all accepted by excel-graph (6 cells it could not decode left pending); the engine passed 924/994 cells
with no crash. A LAMBDA optional parameter (`[y]`) could not be probed: the case builder's file form is refused (T-1274).
Eval delta: probes +1 case, 158 scored cells, 57 pass, 101 new-coverage known failures. Follow-ups: T-1271, T-1272,
T-1273, T-1274, T-1275, T-1276.

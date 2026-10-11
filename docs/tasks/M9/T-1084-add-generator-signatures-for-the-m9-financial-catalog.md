---
id: T-1084
title: "Add generator signatures for the M9 financial catalog functions"
milestone: M9
lane: harness
status: done
priority: P2
size: M
tier: opus
depends_on: [T-0040]
---

## Goal
`rustsheet-gen --milestone M9` can generate formulas for the financial functions (55 in scope): each in-scope catalog
entry with milestone M9 has a full signature (`returns`, `params` with `mode`, `type`, `domain`, `prior`).

## Context
T-0040 cataloged every Excel function, but only the 35 M2 entries carry signatures; the others
are one-line entries (`name`, `category`, `milestone`, `scope`) that the generator skips
(`Catalog.scoped`). Minimum argument counts are already vetted in eval/catalog/excel-functions.tsv,
and tools/tests/test_catalog_arity.py checks every signature against them. Argument modes
(value / aggregate / range / criteria) decide coercion, so take them from oracle evidence
(excel-probe skill), not from the documentation alone. Functions marked `oracle: none` need
another evidence source before a generated suite can score them. One of the per-category tasks split from the M9 signature work; categories here: financial.

## Acceptance criteria
- [x] Every `scope: in` catalog entry with `milestone: M9` and category financial (and no `oracle: none`) has a signature
  whose required-argument count matches excel-functions.tsv.
- [x] A seeded `gen-m9` sample (`rustsheet-gen --milestone M9`) builds, and Excel for the web
  accepts every generated workbook (no arity-invalid calls).
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing the functions; filling the generated suite's oracle values (own task).

## Outcome
All 55 in-scope M9 financial entries now carry signatures in eval/catalog/functions.yaml (none is `oracle: none`).
test_catalog_arity checks each required count against excel-functions.tsv, and the new
test_generator_emits_every_m9_financial_function checks that the generator emits all 55.
Modes come from the new probe probes/fin-argument-modes (386 cells, excel-graph 2026-10-11; financial.md "Financial
function argument modes"). Every scalar is `value`: all of them lift over an array literal, and the core functions also
lift over a range, while the Analysis ToolPak ones are `#VALUE!` over a range. NPV's values are `aggregate`. IRR, MIRR,
XNPV, XIRR and FVSCHEDULE take their cash flows, dates and schedule as `array`.
Soft integer domains keep dates in their required order with disjoint serial windows: issue < first_interest <
settlement < maturity for ACCRINT, issue < settlement < first_coupon < maturity for ODDF*, and last_interest <
settlement < maturity for ODDL*. They also cover periods,
lives, basis and type. The generator now writes a number `enum` unquoted, so frequency is 1, 2 or 4 and not the text
"2" (test_number_enums_are_number_literals).
A seeded sample (`rustsheet-gen --milestone M9 --count 20 --seed 17`, from a scratch catalog holding only the financial
signatures, not committed; the catalog was
functions.yaml filtered to the entries with `category: financial` and `params`, passed with `--catalog`) made 800 formulas calling all 55 functions. excel-graph accepted all 20 workbooks
(0 failures), and the engine passed 700/800 cells with no abort.
Eval delta: probes +1 case, 386 cells, 80 pass, 306 new known failures (T-1372 lifting 274 cells; T-1373 NPV and
FVSCHEDULE 6; T-1374 bond and depreciation values; the annuity and solver ulps belong to T-0901 and T-0916).
Regressions 0.
Follow-ups: T-1372, T-1373, T-1374.

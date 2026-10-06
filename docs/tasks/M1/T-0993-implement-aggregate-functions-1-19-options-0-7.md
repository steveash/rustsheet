---
id: T-0993
title: "Implement AGGREGATE (functions 1-19, options 0-7)"
milestone: M1
lane: functions-math
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
AGGREGATE evaluates function numbers 1-19 with the ignore options 0-7, as Excel does.

## Context
The engine returns #NAME? for AGGREGATE. First evidence: probes/aggregate-overflow A27-A29 (T-0192): AGGREGATE(9,6,F1:F3) and AGGREGATE(1,6,F1:F2) overflow to #NUM!, AGGREGATE(9,6,F1:F3,NA()) is #N/A. Probe the other function numbers, options and the array form (14-19) with the excel-probe skill before implementing; reuse the SUBTOTAL and statistics code.

## Acceptance criteria
- [x] probes/aggregate-overflow A27-A29, B27-B29, C27-C29 pass; further probe cases cover each function number and option.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
AGGREGATE implemented (`functions/aggregate.rs`): it gathers the numbers its option keeps (nested
SUBTOTAL/AGGREGATE cells, errors, hidden rows) and hands them to the existing SUM/LARGE/PERCENTILE/...
functions, so each statistic keeps one definition. The reference form reads references and unions only
(`reference_areas`, now pub(crate)); the positional walk `for_each_position_in_range` skips cells
unevaluated. The `aggregate` name was added to languages.json (en/it/fr/de/es) and language.bin regenerated
with tools/upstream/generate_language (it reproduced the old file byte for byte first). New probes
probes/aggregate-grid (217 cells) and probes/aggregate-args (53 cells), oracle excel-graph; semantics in
docs/excel-semantics/numbers.md "AGGREGATE". Eval: probes known failures 233 -> 231 (fixed 9: aggregate-overflow
A27:C29; 263 new cells pass, 7 new known failures). Follow-ups: T-1034 (LARGE/SMALL k), T-1038 (array
function_num/option/k), T-1036 (hidden/filtered rows and nested calls, unprobed), T-1037 (VAR.P last digit,
grid B31/C31). Hidden rows are read once per call (review: a per-cell row-style scan was quadratic);
unit test test/test_aggregate.rs covers hidden rows and nested SUBTOTAL cells.

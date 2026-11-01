---
id: T-3653
title: "PERCENTOF: probe and match which error comes out for error, text, boolean and coerced arguments"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
PERCENTOF gives Excel's error (and the right value) for every kind of argument: errors in either argument and in ranges, text, numeric text, booleans, blanks, arrays and mismatched shapes.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md). It names a class of input that fails, never a case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill, and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the class, record that in the Outcome and close the task.

The holdout themes note reports PERCENTOF returning the wrong error for error and coerced arguments. Done:
T-0192 (overflow and error precedence of sums, includes PERCENTOF), T-1336 (near-zero snap), T-1325 (GROUPBY
use). Open: T-2862 and T-3494 (argument-kind coverage cells only). Not swept: an error in data_subset versus in
data_all (which wins), errors inside each range, direct text and numeric text versus the same in a cell, direct and
referenced booleans, data_all summing to 0 (#DIV/0!) with and without an error present, a subset not contained in
data_all, arrays of different shapes, and a 3-D or union reference. Code: `fn_percentof` in
functions/math_and_trigonometry/mathematical.rs; compare with SUM's argument walker it should share.

## Acceptance criteria
- [x] A probe case (excel-graph) covers the argument kinds above.
- [x] The engine matches every probed cell; numbers.md states PERCENTOF's coercion and error order.
- [x] Every quirk found is written up in docs/excel-semantics/numbers.md in the same PR, with its probe cells cited.
- [x] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- GROUPBY/PIVOTBY use of PERCENTOF.

## Outcome
Probed 103 cells in probes/t3653-percentof-argument-kinds and 12 in probes/t3653-percentof-3d (excel-graph,
2026-11-01). Coercion already matched SUM. Errors did not: Excel reads both arguments in one walk with SUM's
ranking, so a direct error (an error value, a scalar computed from one, non-numeric typed text) wins in argument
order. An error inside a reference or array, in either argument, is `#NUM!` (ahead of a zero total), and a 3-D
reference is a direct `#VALUE!`. The new `Model::percentof_sums` (sum.rs) walks both arguments with one shared
`AggregateWalk` (new `AggregateWalk::for_numbers` constructor, also used by `for_each_aggregate_number`).
`fn_percentof` divides through the existing `percent_of`. numbers.md: "PERCENTOF: an error inside a reference
or array is #NUM!". Unit tests in test/test_fn_percentof.rs.
Eval delta: the two new probe cases pass 103/103 and 12/12; probes/percentof-snap-overflow-union still passes 15/15;
no regressions.

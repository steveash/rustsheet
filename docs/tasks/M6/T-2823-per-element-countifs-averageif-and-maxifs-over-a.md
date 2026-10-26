---
id: T-2823
title: "Per-element COUNTIFS, AVERAGEIF and MAXIFS over a lifted OFFSET (T-2065 probes)"
milestone: M6
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
COUNTIFS, AVERAGEIF and MAXIFS whose range is an array of references (a lifted OFFSET) spill one result per element,
as Excel does, so the owned M6 exit cells pass.

## Context
T-2065 (#1944) probed these on excel-graph (2026-10-26) and appended the values to T-2064, which was already done, so
the cells had no open owner (`make exit-owners MILESTONE=M6`, steward run 2026-10-26 18:47 UTC). Excel:
`COUNTIFS(OFFSET(A1,{0;1},0),">0")` spills 1, 1; `AVERAGEIF(OFFSET(A1,0,0,{1;2}),">0")` 1, 50.5;
`MAXIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0")` 1, 100. Rules: docs/specs/SPEC-0014-reference-array-results.md,
docs/excel-semantics/lookup.md, and T-2064's "T-2065 findings". Code: functions/criteria.rs and
functions/statistical/if_ifs.rs (the per-element lifting T-2064 added for other callers).

Owned M6 exit cells (6; known failures in eval/baseline/probes.txt):
- probes/t2065-lift-countifs Sheet1!E1, Sheet1!E2
- probes/t2065-lift-averageif Sheet1!E1, Sheet1!E2
- probes/t2065-lift-maxifs Sheet1!E1, Sheet1!E2

## Acceptance criteria
- [x] The six cells leave eval/baseline/probes.txt (`make eval-baseline`), with no regressions.
- [x] `make exit-owners MILESTONE=M6` no longer lists them; lookup.md says the engine implements the rule.
- [x] `make check` green; reviewers approve.

## Out of scope
- STDEV.S, VAR.S, SUMSQ, AVERAGEA and MAXA over the same form: T-2824.
- MEDIAN, LARGE and SMALL, which do not open on Excel for the web: T-1075.

## Outcome
`Function::per_reference_element` (functions/lookup_and_reference/ref_lift.rs) now lists COUNTIFS (every criteria
range), AVERAGEIF (range and average range, as SUMIF) and MAXIFS (max range and every criteria range, as SUMIFS), each
element charged one walk per criteria pair as SUMIFS is (the existing SPEC-0014 bound DEVIATION); their static shape
comes from `criteria_and_grid_static_result`. AVERAGEIFS and MINIFS are unprobed and stay the value view (lookup.md).
Eval: probes known failures 2631 -> 2625 (the six owned cells), no regressions.
Follow-up: T-2833 probes the argument positions inferred from SUMIF/SUMIFS.

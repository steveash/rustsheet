---
id: T-2318
title: "TEXTJOIN lifts an array ignore_empty (gen-m7 ac67253061bba16a A1137 is #SPILL!)"
milestone: M7
lane: functions-text
status: done
priority: P2
size: S
tier: opus
campaign: visible
depends_on: []
---

## Goal
TEXTJOIN lifts an array ignore_empty (and an array delimiter given through a LAMBDA parameter) to one result per element, as Excel's #SPILL! (a blocked multi-cell result) shows, instead of #N/IMPL!.

## Context
Found by the T-2225 triage of eval/suites/gen-m7 (38 cases, 10,991 Excel-scored cells, excel-graph). Quirk entry: docs/excel-semantics/arrays.md "TOROW, TOCOL, SORT and TEXTJOIN lifted arguments (gen-m7)".

Evidence: `LAMBDA(y,RIGHT(TEXTJOIN(y,y,y),2))(UNIQUE(Data!C3:D4))` is `#SPILL!` in Excel (the result is an array and its spill is blocked), ours `#N/IMPL!` (ac67253061bba16a A1137). Probe the element values (`TEXTJOIN(",",{TRUE;FALSE},"a","")`) before choosing the rule if arrays.md does not have it. Code: `fn_textjoin` in functions/text/concatenate.rs.

Visibly-off cells (gap triage 2026-10-21, decision 0083): 1 cells, every one with a wrong type, error, text or value (none is a last-digit difference).

Owned gen-m7 cells (T-2225 triage, excel-graph, 2026-10-21): 1 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas. One line per formula: case, anchor, the anchor formula, Excel vs ours at the first failing cell, then every failing cell (sheet Calc; a range is consecutive cells of one row).

- gen-m7/ac67253061bba16a.case.json A1137 `=LAMBDA(y,RIGHT(TEXTJOIN(y,y,y),2))(UNIQUE(Data!C3:D4))`: A1137 Excel #SPILL!, ours #N/IMPL!; cells (1): A1137

## Acceptance criteria
- [x] Visible parity: the 1 gen-m7 cells listed under Context have Excel's value, type, error and text: these 1 known-failure lines disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [x] The docs/excel-semantics entry cited above states the rule and its "Ours:" line names this task's change.
- [x] Unit tests cover the rule on a small hand-written input (one error or edge case and one normal case that must keep passing).
- [x] `make check` green; reviewers approve.

## Out of scope
- -

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off (a wrong type, error, text or value), so this task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value (an expected 0 exactly 0); the visible-parity criterion under Acceptance criteria states it.

## Outcome
TEXTJOIN is in the SPEC-0011 lifting table (`[WholeByCaller, Lift]`, repeat `WholeByCaller`, max
255): an array or range `ignore_empty` gives one join per element, with the delimiter and texts
read once and at most two joins built (one per TRUE/FALSE). The hand-written static-analysis arms
are gone; the catalog lists the delimiter as `mode: array`. New probe
probes/textjoin-ignore-empty-lift (excel-graph, 132 cells, all pass). Eval delta: gen-m7 known
failures -1 (ac67253061bba16a A1137), probes +1 case / 132 cells / 0 failures, no regressions.

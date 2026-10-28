---
id: T-2543
title: "IMSUB and IMDIV: an empty value from IFERROR or IFNA is #N/A in ours, not 0 (gen-m9, 1 cell)"
milestone: M9
lane: functions-engineering
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: semantics
---

## Goal
IMSUB and IMDIV treat an empty value passed through IFERROR or IFNA as 0, as they treat an empty cell, instead of returning `#N/A`.

## Context
Gap triage of the T-2258 gen-m9 engineering sweep. In the engine `IMSUB("1",IFERROR(<empty cell>,"x"))`, `IMDIV("1",IFERROR(<empty>,"x"))`, `IMSUB(IFERROR(<empty>,"x"),"1")` and `IMSUB("1",IFNA(<empty>,"x"))` are `#N/A`, while `IMSUB("1",IF(TRUE,<empty>))` is "1" and IMPOWER, IMSUM and IMABS of the same value work: the two-argument path of IMSUB/IMDIV mishandles the empty value. Excel's cell gen-m9/2b0324e2de2e7b74 A63 `IMSUB((HYPERLINK(Data!B9)&Data!$E7),IFERROR(Data!D1,..))` ("a*abc" against an empty D1) is the first argument's `#NUM!` (ours `#N/A`). engineering.md "T-1625 probes": a blank cell argument is 0 in IMSUB. Excel's value for `IMSUB("1",IFERROR(<empty>,"x"))` is unprobed (expected "1"): add it to a probe case.
Code: `fn_imsub`, `fn_imdiv` in complex.rs.

Owned gen-m9 cells (T-2258 triage, excel-graph, 2026-10-23): 1 known-failure lines in eval/baseline/gen-m9.txt, 1 formulas, 1 visibly off.
Examples:
- gen-m9/2b0324e2de2e7b74 Calc!A63 `=IMSUB((HYPERLINK(Data!B9)&Data!$E7),IFERROR(Data!D1,FIXED(Data!D$7,3,TRUE)))`: Excel #NUM!, ours #N/A
Cells:
- gen-m9/2b0324e2de2e7b74.case.json (1): Calc!A63

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2258): these 1 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probe of IMSUB and IMDIV over IFERROR(<empty>,..) is oracle-filled and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other T-2258 triage families: each has its own task (T-2258 Outcome).
## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2258): every cell this task owns has the wrong type, error, text or value (none is a last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria states it.

## Outcome
`complex_pair_of` (IMSUB, IMDIV) raised `#N/A` for any `CalcResult::EmptyArg`, which is also the empty value IFERROR/IFNA hand on for a blank cell. It now checks the argument nodes: only an argument left out of the formula is `#N/A`; IFERROR's empty value is 0. New probe `probes/eng-imsub-imdiv-iferror-empty` (10 cells, excel-graph) passes. Eval delta: gen-m9 -1 known failure (A63), no new known failures.

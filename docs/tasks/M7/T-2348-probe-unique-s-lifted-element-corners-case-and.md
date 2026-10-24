---
id: T-2348
title: "Probe UNIQUE's lifted-element corners: case and -0 entries, call errors, INDEX of other empty values"
milestone: M7
lane: functions-lookup
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-2300]
campaign: visible
---

## Goal
Excel evidence for the corners of UNIQUE's lifted-element rule that T-2300 implemented by inference, and the engine matching it.

## Context
From T-2300's Excel-fidelity review. `CallResult::ArrayFirst` (functions/lift/driver.rs, `Later`, `all_elements_equal`) decides "entries all equal" by `ArrayNode` equality. It treats an element whose call returns an error as one that stays and is not the kept first element. INDEX of any empty value is `#VALUE!` (`scalar_as_one_element_array` in lookup_and_reference/index.rs). Only UNIQUE's empty element is evidenced (probes/m7-lift-take-drop-sortby L25, probes/m7-unique-lift-elements). Unprobed:
- text entries that differ only in case, `UNIQUE(D1:D3,FALSE,{"a";"A"})` (ours: they differ, because `ArrayNode` equality is case-sensitive), and `{0;-0}` (ours: equal);
- a call error from a non-error entry before a good one, `UNIQUE(D1:D3,FALSE,{"abc";FALSE;TRUE})` (ours: `#VALUE!`, "x", empty);
- INDEX of an empty value from another source, `INDEX(IFERROR(B1,1),1,1)` and `LET(x,IFERROR(B1,1),INDEX(x,1,1))` with B1 blank (ours `#VALUE!`; arrays.md "UNIQUE's and SORTBY's lifted elements" marks it UNVERIFIED).
Write the probe case per the excel-probe skill (ROWS and per element ISBLANK, `&"|"`, TYPE and SUM, as m7-unique-lift-elements does), fill it on excel-graph, and fix whatever differs.

gen-m7 cells (T-2254 triage, excel-graph, 2026-10-22): a 2-D exactly_once whose entries are all equal. `UNIQUE(Data!C2:E10,("a"<>Data!$B4),
EXACT(TRANSPOSE(Data!C10:E12),"1.5TRUE"))` has by_col TRUE and a 3x3 exactly_once of FALSE everywhere (checked in ours); Excel spills 100
(the call's top-left) down column A and 0 (empty elements) in B1284:C1286 (gen-m7/2c156ba98ef0220a.case.json A1284). arrays.md says that when every entry
is equal each element is its call's top-left, and that a 2-D broadcast or 2-D by_col keeps only the top-left element; here a uniform 2-D
array keeps the first element of each row (or column). Ours: 100 in all nine. Add 2-D uniform cases to the probe (`UNIQUE(D1:D3,FALSE,
{FALSE,FALSE;FALSE,FALSE})`, a 2x3 and a 3x2, by_col TRUE and FALSE) to tell "first per row" from "first column" and from by_col's
direction, and fix ours to match. Owned: 6 known-failure lines in eval/baseline/gen-m7.txt, 1 formulas.
- gen-m7/2c156ba98ef0220a.case.json A1284 `=UNIQUE(Data!C2:E10,("a"<>Data!$B4),EXACT(TRANSPOSE(Data!C10:E12),("1.5"&"TRUE")))`: B1284 Excel 0.0, ours 100.0; cells (6): B1284:C1284, B1285:C1285, B1286:C1286

## Acceptance criteria
- [ ] A probe case with these cells, filled from excel-graph; its findings recorded in arrays.md (the UNVERIFIED line resolved).
- [ ] Every cell of it passes, or each failing cell has its own task.
- [ ] Visible parity: the 6 gen-m7 lines listed under Context (T-2254 triage) disappear from eval/baseline/gen-m7.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2254): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probe `probes/m7-unique-lift-corners` (21 questions, excel-graph, 2026-10-23) filled; findings in arrays.md "UNIQUE's and SORTBY's lifted elements": uniform entries spanning several columns keep the call's top-left only in the first column (other columns empty), independent of by_col; text entries are `#VALUE!` whatever their case; `{0;-0}` is equal; call errors stay and the first good element is kept; INDEX of an empty value from IFERROR is `#VALUE!` (already matched). Engine: `empty_past_first_column` in lift/driver.rs. Eval: gen-m7 known failures 45 -> 39 (the 6 owned lines), the new probe passes in full, no regressions. The rule is UNIQUE-only (`CallResult::ArrayFirstColumn`); SORTBY is unchanged, and its uniform 1xN sort_order and an error element past column one are UNVERIFIED in arrays.md, added to T-2349.

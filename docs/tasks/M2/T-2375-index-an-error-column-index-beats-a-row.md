---
id: T-2375
title: "INDEX: an error column index beats a row index below 1 (INDEX(B2:E6,-0.5,#N/A) is #N/A)"
milestone: M2
lane: functions-lookup
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
INDEX returns an error in its column argument even when its row argument is below 1, as Excel does.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `INDEX(Data!B2:E6,TAN(-0.5),MATCH("1",Data!B5:C8,1))`: the row -0.546 truncates below 1 and the column is `#N/A` (MATCH over
a 2-D range). Excel gives `#N/A`; ours checks the row first and gives `#VALUE!`, which MMULT passes on. Same family as the known
`INDEX(A5:B7,NA(),3)` = `#N/A` (an error row wins over an out-of-range column): argument errors come before index validation.
Rule: docs/excel-semantics/lookup.md, the paragraph after Rule 2 of the MATCH match_type entry (added by this triage). Code:
`fn_index` in functions/lookup_and_reference/. Check the reference and array forms alike.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/372a676d47aa1427 Calc!A371 | `=MMULT(Data!A1,INDEX(Data!B2:E6,TAN(Data!A$5),MATCH("1",Data!B5:C8,1)))` | #N/A | #VALUE! |

## Acceptance criteria
- [ ] A unit test: `INDEX(range,-0.5,NA())` and `INDEX({1,2;3,4},-1,NA())` are `#N/A`; `INDEX(range,-0.5,1)` stays `#VALUE!`.
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MATCH over a 2-D range (already `#N/A`).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
`index_numbers` now reads the row and column cast errors before range-checking either for a negative; the area_num order is unchanged. gen-m5 372a676d47aa1427 Calc!A371 left eval/baseline/gen-m5.txt (1 fixed, 0 regressions). Unit test `test_index_error_column_beats_negative_row`.

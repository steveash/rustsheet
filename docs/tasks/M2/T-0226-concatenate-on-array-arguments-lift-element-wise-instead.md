---
id: T-0226
title: "CONCATENATE on array arguments: lift element-wise instead of #VALUE! (2 gen-m2 known failures)"
milestone: M2
lane: functions-text
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`CONCATENATE` given a range or array argument returns an array (each element is the
concatenation of the matching elements, scalars and length-1 dimensions broadcast), as Excel
does. Today it returns `#VALUE!`. `CONCAT` is different (it joins a whole range into one string)
and does not change.

## Context
Found by the gap triager on 2026-10-05 (gen-m2 cells that T-0208 turned into `#SPILL!` expectations
and that no task owned). Both cells sit in column A of the Calc sheet with the next row occupied, so
a multi-cell result shows `#SPILL!`. A scalar `#VALUE!` from CONCATENATE would have stayed
`#VALUE!`, so the `#SPILL!` shows that CONCATENATE returned an array.
- `gen-m2/207a5753da48c7e0` Calc!A297
  `=(AVERAGE(Data!D3:D4,Data!D1)=CONCATENATE(INDEX(Data!C11:C12,0),("#N/A"&Data!D4)))`
  (INDEX returns the 2-row column C11:C12): expected `#SPILL!`, actual `#VALUE!`.
- `gen-m2/b4392a6514a36ade` Calc!A348 `=TRIM(CONCATENATE(INDEX(Data!A7:D11,Data!A8,3)))`
  (Data!A8 is 0.3, which truncates to row 0, so INDEX returns the column C7:C11): expected
  `#SPILL!`, actual `#VALUE!`.

Checked in a scratch case: with room to spill, `INDEX(...)`, `=Data!C11:C12&"x"` and
`=TRIM(Data!C7:C11)` already spill in the engine, while `=CONCATENATE(Data!C11:C12,"x")` gives
`#VALUE!`. So the `&` operator and TRIM lift already, and only CONCATENATE is missing.

Semantics: docs/excel-semantics/arrays.md "NOT and CONCATENATE lift over array arguments".
Code: `fn_concatenate` in `crates/rustsheet-engine/src/functions/text/common.rs` reads each
argument with `get_string`. The same file already has the text broadcasting helper
(`broadcast_text`, plus the `text_arg` argument reader that EXACT/LEFT/MID use). Reuse them and do
not write a new helper. Keep the scalar path's results exactly as they are.

## Acceptance criteria
- [ ] `F	gen-m2/207a5753da48c7e0.case.json	Calc!A297` and
      `F	gen-m2/b4392a6514a36ade.case.json	Calc!A348` disappear from eval/baseline/gen-m2.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [ ] Unit tests: `CONCATENATE({"a";"b"},"x")` is `{"ax";"bx"}`; `CONCATENATE(A1:A2,B1:B2)`
      pairs elements; scalar calls are unchanged; `CONCAT(A1:A2)` still joins into one string.
- [ ] docs/excel-semantics/arrays.md entry gets an `Implemented in:` line.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Element-wise error propagation over arrays in operators (T-0129).
- Mismatched array shapes beyond size-1 broadcasting (keep whatever `broadcast_text` does
  and file a probe task if a case needs it).

## Outcome
CONCATENATE now lifts over range/array arguments via `text_arg` + `broadcast_text`; static analysis reports an array result. gen-m2 known failures 1518 -> 1514 (4 fixed, the 2 targets plus 2 more), no regressions.

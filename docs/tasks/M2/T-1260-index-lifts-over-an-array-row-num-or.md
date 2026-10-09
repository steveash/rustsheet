---
id: T-1260
title: "INDEX lifts over an array row_num or column_num in a dynamic-array formula (#N/IMPL! today)"
milestone: M2
lane: functions-lookup
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
INDEX with an array row_num or column_num (an array literal or a function returning an array, e.g. CHOOSEROWS) lifts element-wise in a dynamic-array formula, as Excel does, instead of `#N/IMPL!`.

## Context
T-1239's gen-m6 suite (excel-graph sidecars, 2026-10-09): gen-m6/2c1a6e93a1974f48 Calc!A25
`=(INDEX(Data!C2:D9,CHOOSEROWS(Data!D7:E11,4))&Data!B5)`: CHOOSEROWS gives the 1x2 array {0, FALSE} (D10 blank, E10 FALSE);
each element as a lone row index into a 2-D range is `#REF!` (lookup.md "INDEX with one index into a 2-D range is #REF!"), and
Excel's cell is `#REF!`. The engine gives `#N/IMPL!` for any array index (`INDEX(Data!C2:C9,{1,2})` too).
T-0294 covers the legacy (implicitly intersected) form, where Excel takes the first element; this task covers the
dynamic-array lift. Probe first (excel-probe skill): `INDEX(A1:A5,{1,3})`, `INDEX(A1:B5,{1;2},{2;1})`, `INDEX(A1:B5,{1,2},1)`,
`INDEX({1,2;3,4},{1;2},2)`, `SUM(INDEX(A1:A5,{1,3}))`, an error element and a 0 element, and whether the result elements are
references (`ROWS`, `ISREF`). Reuse the existing lift helpers, as T-1212/T-1218 did (directive 3). Code: `fn_index`
(crates/rustsheet-engine/src/functions/lookup_and_reference/mod.rs).

## Acceptance criteria
- [ ] An excel-graph probe case with the cells above passes; lookup.md records the rule.
- [ ] gen-m6/2c1a6e93a1974f48 A25 passes in `./target/release/rustsheet-eval show <case>`, with no new failing gen-m6 cell and
      no new known failures in eval/baseline/.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The legacy first-element form (T-0294); INDEX of a scalar (T-1242).

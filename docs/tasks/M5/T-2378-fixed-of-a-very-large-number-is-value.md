---
id: T-2378
title: "FIXED of a very large number is #VALUE! (FIXED(99^100)): find the bound"
milestone: M5
lane: functions-text
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
FIXED (and DOLLAR, if the probe shows the same bound) returns `#VALUE!` past Excel's bound, so `FIXED(99^100)` matches.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `FIXED(99^100)` (3.66E+199) is `#VALUE!` in Excel; ours prints 15 digits, 185 zeros and ".00" with commas, 269 characters.
Excel prints `DOLLAR(1.0000001234567907E+30,-2)` and a 103-character `FIXED(ASINH(123456790.123),100)`, so the bound is either the
result's length (255 characters?) or the magnitude. Observation: docs/excel-semantics/text.md "FIXED of a very large number is
#VALUE!" (added by this triage). Needs a probe first (`excel-probe` skill, excel-graph): FIXED and DOLLAR of 1E+100, 1E+200,
1E+250, 1E+300, 9.99E+307 with decimals 0, 2 and -2 and no_commas TRUE/FALSE, plus values whose comma form is 254, 255, 256
characters (for example 1E+190 vs 1E+195 with commas) and `FIXED(1,127)` vs `FIXED(1E+125,127)`. Code: `fn_fixed`, `fn_dollar`
in functions/text/ (string_format.rs) and the shared `significant_15` digits.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 1 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/55afc805e00a1509 Calc!A392 | `=FIXED(((Data!$A3+Data!D3)^100))` | #VALUE! | "36,603,234,127,322,900,000,...,000.00" (269 characters) |

## Acceptance criteria
- [ ] A probes case with the cells above, oracle-filled (excel-graph), passes in full; text.md states the bound.
- [ ] This 1 known-failure line disappears from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 15-digit cut (T-2148, done).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

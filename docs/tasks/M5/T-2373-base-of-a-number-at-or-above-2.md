---
id: T-2373
title: "BASE of a number at or above 2^53 is #NUM! (BASE(ROUNDUP(2^53,-6),10,5))"
milestone: M5
lane: functions-math
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-2289]
campaign: visible
---

## Goal
BASE returns `#NUM!` for a number past Excel's upper bound (2^53, the exact edge from T-2289), instead of printing its digits or i64::MAX's.

## Context
Gap triage of gen-m5 round 2 (T-2252, excel-graph oracle, 2026-10-22). `base_text` (functions/math_and_trigonometry/mathematical.rs) truncates with `number.trunc() as i64` and has no upper
bound, so 9007199255000000 prints in full and anything above 2^63 saturates. Excel refuses both cells below with `#NUM!`.
Rule: docs/excel-semantics/numbers.md "BASE of zero with min_length 0 is empty; BASE past 2^53 is #NUM!" (added by this triage).
T-2289 probes the exact edge (2^53-1, 2^53, 2^53+2, 2^63, 1E+300); use its cells for the boundary and check where the bound
sits against the sign check and the `min_length` check (numbers.md "BASE min_length is at most 255").

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 2 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/55afc805e00a1509 Calc!A101 | `=BASE(ROUNDUP(2^53,-6),10,5)` | #NUM! | "9007199255000000" |
| gen-m5/08eafc9e61602bcb Calc!A98 | `=BASE(SUM(PRODUCT(Data!D3:D8,Data!$C6),Data!E6:E6,IFERROR(Data!B2,Data!B12)),34)` | #NUM! | "3TDTK1V8J6TPP" |

## Acceptance criteria
- [x] T-2289's probe cells at and around 2^53 pass.
- [x] These 2 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [x] `make check` green; reviewers approve.

## Out of scope
- DECIMAL's bounds.
- BASE of zero (T-2374).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-22, T-2252): every cell this task owns has the wrong type, error, text or boolean, or is a
number further than half a unit in Excel's 15th significant digit, so the task is in the campaign. Its acceptance
criteria above are already the visible-parity stage.

## Outcome
`base_text` returns `#NUM!` for a number of 2^53 or more (edge from T-2289). Eval: 32 known failures fixed (2 gen-m5, 30 probes/base-number-upper-bound), no regressions.

---
id: T-2093
title: "F.TEST and FTEST with a scalar or text argument are #DIV/0!, not #VALUE!"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
F.TEST and FTEST return `#DIV/0!` when either argument is a plain number or a text, as Excel does,
instead of `#VALUE!`.

## Context
Gap triage 2026-10-20. Excel treats a scalar argument as a sample with fewer than two points.
docs/excel-semantics/statistics.md, "Thin distribution functions ... (T-1049)", "Test functions",
already says this: "F.TEST: `#DIV/0!` for fewer than two points on either side, ... scalars, an
empty range or text-only input (`F.TEST("a",B1:B5)`)". No task implemented it.
Failing cells (excel-graph; probes-numeric is not gated and has no baseline file, so read them with
`make eval-show CASE=probes-numeric/<case>.case.json`). Each is Excel `#DIV/0!` and ours `#VALUE!`:
- probes-numeric/t1049-ftest Calc!A29 `F.TEST(B1:B5,3)`, A30 `F.TEST(3,B1:B5)`, A31 `F.TEST(3,4)`,
  A32 `F.TEST("a",B1:B5)`.
- probes-numeric/m9-legacy-ftest Calc!A32 `FTEST(1,B1:B5)`.
Cells that already pass and must keep passing: t1049-ftest A15 `F.TEST(B1:B1,C1:C5)` and A16 (a
one-cell range gives `#DIV/0!`), and A22 `F.TEST(B1:B5,Z1:Z5)` (an empty range).
Code: `fn_f_test` in crates/rustsheet-engine/src/functions/statistical/fisher.rs. The `_ =>` arm of
each argument's match returns `#VALUE!` for any non-range, non-array value. A number or a text
should instead go on to the existing `n1 < 2 || n2 < 2` check, which gives `#DIV/0!`. The simplest
way is to read it as a sample with no numbers. FTEST (`Function::FTestCompat`) shares `fn_f_test`.
Leave a boolean or an error value as it is now: no probe shows what Excel does with
`F.TEST(TRUE,B1:B5)` or `F.TEST(NA(),B1:B5)`.

## Acceptance criteria
- [ ] The five cells above pass in `make eval`. t1049-ftest A15, A16 and A22 still pass, and no
      gated baseline (eval/baseline/*.txt) gains an F line.
- [ ] A unit test covers `F.TEST(3,4)`, `F.TEST("a",{1,2,3})` and `FTEST(1,{1,2,3})`, each `#DIV/0!`.
      The code comment cites the probe cells.
- [ ] statistics.md's T-1049 entry says the scalar rule is implemented (T-2093).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A boolean or error-valued scalar argument (unprobed; file a probe task if needed).
- F.TEST's last digits (T-1927, T-0970).

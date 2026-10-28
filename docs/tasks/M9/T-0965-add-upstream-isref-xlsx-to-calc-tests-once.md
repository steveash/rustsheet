---
id: T-0965
title: "Add upstream ISREF.xlsx to calc_tests once its remaining cells pass"
milestone: M9
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-0223, T-0964]
---

## Goal
The Excel-authored upstream workbook `INFORMATION/ISREF.xlsx` (ironcalc `9add6b67`) is in `crates/rustsheet-xlsx/tests/calc_tests` with every cell passing.

## Context
`build.rs` generates a strict per-workbook test for every file in calc_tests, so a workbook with any failing cell cannot be added (no skips; CLAUDE.md directive 2). T-0940 ported the ISREF-of-name and FILTER if_empty-range fixes and left the workbook out: 10 of its 43 cells fail on D2 (MAX of FILTER, T-0223), D9/J17:L17 (FILTER if_empty array, T-0964) and A19-A26 (LET/LAMBDA variables, T-0963). Recover the file with `git -C <ironcalc clone> show 9add6b67:xlsx/tests/calc_tests/INFORMATION/ISREF.xlsx`.
No dependency on T-0963 (T-0091): T-0223 landed its code, and T-0963 only confirms its five cells once this task adds
the workbook, so T-0963 depends on this task and not the reverse (the two formed a `depends_on` cycle).

## Acceptance criteria
- [x] ISREF.xlsx added; `rustsheet-eval show ironcalc-calc/INFORMATION/ISREF.xlsx` shows 43/43; baseline updated, 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Added `INFORMATION/ISREF.xlsx` (ironcalc 9add6b67) to calc_tests. `rustsheet-eval show` reports 43/43 (Excel-cached values); the T-0223/T-0964/T-0940 fixes had already landed. Baseline `ironcalc-calc.txt` +1 line, 0 regressions; `make check` green.

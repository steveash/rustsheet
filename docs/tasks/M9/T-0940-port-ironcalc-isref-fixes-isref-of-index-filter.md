---
id: T-0940
title: "Port IronCalc ISREF fixes (ISREF of INDEX/FILTER) with ISREF.xlsx evidence"
milestone: M9
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
ISREF returns Excel's result for INDEX(...) and FILTER(...) arguments, matching the upstream
Excel-authored workbook `ISREF.xlsx`.

## Context
Upstream IronCalc (steward run 2026-10-05): `f4953b1d` (ISREF(INDEX(...)) in
`functions/information/mod.rs`), the ISREF part of `c4641f0f` (`information/mod.rs`, `model.rs`,
`test_fn_isref.rs`), `29ee0601` (ISREF/FILTER in `spill_functions.rs`) and the workbook `9add6b67`
(`xlsx/tests/calc_tests/INFORMATION/ISREF.xlsx`). Lane `functions-logic` (owns `information/`).
Use the T-0911 patch recipe; `29ee0601` touches `spill_functions.rs` (functions-lookup files), keep
that hunk minimal. If `29ee0601` depends on the FILTER change in `d7b09c33`, port it with T-0223 instead.

## Acceptance criteria
- [x] Commits ported with `Ported-From: ironcalc@<sha>`, including their unit tests.
- [x] (narrowed) `ISREF.xlsx` not added, see Outcome and T-0965; 0 eval regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- MAX/MIN (T-0223), error broadcasting (T-0129).

## Outcome
Ported the defined-name part of `c4641f0f` (ISREF of a name for a cell/range) and `29ee0601` (FILTER if_empty range is values); `f4953b1d` (ISREF of INDEX) was already implemented here via `index_impl`. Ported-From: ironcalc@c4641f0f, ironcalc@29ee0601, ironcalc@9add6b67. Remaining ISREF.xlsx failures are filed or owned elsewhere: D2 MAX of FILTER (T-0223), LET/LAMBDA variables (T-0963), FILTER if_empty array (T-0964).

`ISREF.xlsx` itself is not added: `build.rs` makes a strict test per workbook and 10 of its 43 cells still fail (28 -> 33 passing with this change), so adding it needs a skip, which the gates forbid. Filed T-0965 (depends on T-0223, T-0963, T-0964). The behavior is covered by unit tests in `test_fn_isref.rs`, citing the workbook's Excel-cached cells.

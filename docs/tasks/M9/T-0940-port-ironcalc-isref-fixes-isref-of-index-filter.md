---
id: T-0940
title: "Port IronCalc ISREF fixes (ISREF of INDEX/FILTER) with ISREF.xlsx evidence"
milestone: M9
lane: functions-logic
status: in-progress
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
- [ ] Commits ported with `Ported-From: ironcalc@<sha>`, including their unit tests.
- [ ] `ISREF.xlsx` added to `crates/rustsheet-xlsx/tests/calc_tests`; its cells pass or are baselined as new coverage; 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- MAX/MIN (T-0223), error broadcasting (T-0129).

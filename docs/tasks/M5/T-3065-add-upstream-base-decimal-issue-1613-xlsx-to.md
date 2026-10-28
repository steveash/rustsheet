---
id: T-3065
title: "Add upstream BASE_DECIMAL_issue_1613.xlsx to calc_tests"
milestone: M5
lane: functions-math
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Upstream's `MATH_AND_TRIGONOMETRY/BASE_DECIMAL_issue_1613.xlsx` (ironcalc `a12a8b16`) is in `crates/rustsheet-xlsx/tests/calc_tests`, and every one of its cells passes.

## Context
Upstream triage, 2026-10-28. In `a12a8b16`, "FIX: BASE returns #NUM! when min_length exceeds 255", IronCalc adds the cap that we already have: `BASE_MAX_MIN_LENGTH` in `math_and_trigonometry/mathematical.rs`, evidence probes/base-min-length, numbers.md. So there is no code to port. The commit also adds a test workbook. Recover it with `git -C target/upstream-ironcalc show a12a8b16:xlsx/tests/calc_tests/MATH_AND_TRIGONOMETRY/BASE_DECIMAL_issue_1613.xlsx`. First check that it is Excel-authored, meaning the workbook's app.xml is from Microsoft Excel. If it is not, close this task as moot. `build.rs` makes a strict test per workbook, so add the workbook only if every cell passes (no skips).

## Acceptance criteria
- [ ] Workbook added with `Ported-From: ironcalc@a12a8b16`, all cells pass and the baseline is updated with 0 regressions; or the task is closed with the reason (not Excel-authored).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

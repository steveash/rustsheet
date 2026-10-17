---
id: T-1733
title: "Displayed text: mm next to ss with no h is minutes, not months"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`mm` directly before `:ss` (or after `h:`) is minutes even when the code has no hour, as Excel reads it.

## Context
Code `mm:ss`: Excel 1000 "00:00", 123456.789 "56:10", 1.5 "00:00"; ours "09:00", "01:10", "01:00" (months).
71 of 80 text cells fail. Also check `mm:ss.0`/`mm:ss.00` once T-0981 lands (fractional seconds).
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

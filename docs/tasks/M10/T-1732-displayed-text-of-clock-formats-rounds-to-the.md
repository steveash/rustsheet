---
id: T-1732
title: "Displayed text of clock formats rounds to the shown second before splitting the day (0.9999999999 is 0:00)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Clock and date formats round the serial to the shown precision (whole seconds, or the fractional digits shown) before splitting it into day, hour, minute and second, as Excel does.

## Context
0.9999999999 (23:59:59.99999) under `h:mm` is "0:00" in Excel (ours "23:59"), `h:mm:ss` "0:00:00", `hh:mm:ss` "00:00:00",
`[h]:mm:ss` "24:00:00" (ours "24:59:60"), `h:mm:ss AM/PM` "12:00:00 AM" (ours "11:59:60 PM"),
`yyyy-mm-dd` "1900-01-01" (ours "1900-01-00"), `dddd` "Sunday" (ours "Saturday"), `d \d\a\y\s h:mm`
"1 days 0:00", `[m]:ss` "1440:00" (ours "1440:60"), `dd/mm/yy` "01/01/00". Also 45658.999988426. A seconds field
of 60 must never appear.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

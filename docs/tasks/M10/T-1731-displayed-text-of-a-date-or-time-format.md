---
id: T-1731
title: "Displayed text of a date or time format over a serial outside 0..2958465.99999 is 255 #"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A date or time format over a serial Excel cannot show as a date (negative, or past 2958465.99999 = 9999-12-31 23:59:59) displays 255 `#`, as Excel's text does.

## Context
Every date and clock-time code in gen-m10 (`m/d/yyyy`, `d-mmm-yy`, `mmm-yy`, `dddd`, `yyyy-mm-dd`, `h:mm`,
`h:mm:ss`, `h:mm AM/PM`, `[$-409]mmmm d, yyyy`, `e`, ...; ~450 failing text cells) and elapsed `[h]`/`[m]`/`[s]`
codes over 1234567890 or a negative value: Excel shows 255 `#` (the report truncates them), ours `#VALUE!`
(dates) or a number (`h:mm` of 1234567890 "0:00", `[h]:mm` "29629629360:00"). T-1711 has the one probe cell for
a negative date; this is the general rule, also for elapsed codes and values above the last date. Check with the
evidence whether `[h]` of a large positive value is `#` because the serial is past the last date.
Evidence: suite gen-m10 (excel-graph, 2026-10-16, T-1611): `make eval` report, "Displayed text" section, which lists each failing text cell (Excel vs ours). Text cells are scored apart from values and are not baselined (decision 0068); the TEXT() cells on sheet T that use the same codes are value known failures in eval/baseline/gen-m10.txt. Formatter: crates/rustsheet-engine/src/formatter/ (shared by `Workbook::formatted` and TEXT). docs/excel-semantics/number-formats.md.

## Acceptance criteria
- [ ] The gen-m10 text cells under these codes pass in the report's "Displayed text" section (or the remaining ones are filed); the matching TEXT() cells on sheet T leave eval/baseline/gen-m10.txt; no regressions.
- [ ] number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

---
id: T-1366
title: "TEXT: a section with two elapsed tokens ([h]:[mm]) is #VALUE!"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
TEXT with a format section holding two elapsed tokens is #VALUE!, for any value and in both date systems, as Excel does.

## Context
probes/negative-date-sections-1900 and -1904 A9 (`TEXT(-1.5,"[h]:[mm]")`), A21 (`TEXT(1.5,"[h]:[mm]")`) and A22
(`TEXT(0.5,"[mm]:[ss]")`) are #VALUE! in Excel (dates.md "Negative serials under an explicit negative section", T-1105).
The engine formats the positive ones (A21, A22 and the 1904 A9 are known failures). The `ParsePart::Date` arm of
formatter/format.rs (or the parser) needs the check. Also covers A23 `TEXT(-1.5,"[ss] AM/PM")` = #VALUE! in 1904
only if a probe of the positive value shows the format itself is invalid; otherwise leave A23 to T-1330.

## Acceptance criteria
- [ ] probes/negative-date-sections-1900/-1904 A9, A21, A22 pass; no regression in poi ElapsedFormatTests.
- [ ] Probe `TEXT(1.5,"[ss] AM/PM")` and record the result in dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1904 epoch (T-1330).

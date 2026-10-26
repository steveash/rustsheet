---
id: T-1366
title: "TEXT: a section with two elapsed tokens ([h]:[mm]) is #VALUE!"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
campaign: semantics
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
Careful: a bare `h` after `[h]` acts as elapsed too (poi ElapsedFormatTests A10/A40, `"It was "[h]" [yes, "h"] hours"`)
and must stay valid, so the check is on two bracketed elapsed tokens. Also probe the T-1105 extrapolations
(T-1105 review): `TEXT(-1.5,"[h]_)")` and `TEXT(-1.5,"*-[h]")` in 1900 (ghost/spacer count as literals in
`negative_serial_allowed`) and a conditional section routing a negative to a date section (`TEXT(-2,"[<-1]d;0")`).

## Acceptance criteria
- [ ] probes/negative-date-sections-1900/-1904 A9, A21, A22 pass; no regression in poi ElapsedFormatTests.
- [ ] Probe `TEXT(1.5,"[ss] AM/PM")` and the extrapolations above; record the results in dates.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1904 epoch (T-1330).

## Visible-parity stage
Decision 0083: a wrong TEXT or displayed-text result is visible to every user of the format, so this task is in the campaign. Its acceptance criteria are already the visible-parity stage.

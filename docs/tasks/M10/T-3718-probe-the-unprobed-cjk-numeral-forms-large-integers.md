---
id: T-3718
title: "Probe the unprobed CJK numeral forms (large integers, padded elapsed, myriad zero gap, General with literals)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence, and the matching implementation, for the CJK numeral forms T-3532 left unprobed.

## Context
T-3532 implemented numeral-system digits (formatter/numerals.rs). number-formats.md "Numeral-system digits" lists these as unprobed: integers of 10^12 and more (reached through elapsed counts; we print them digit by digit, `MAX_SPELLED`), a padded elapsed count under 10 (`[$-1B000409][hh]` on 0.2; we print "〇四"), a zero gap across a myriad under bytes 30 and 31 (`[$-1E000409]General` on 100005 and 100000005; we write one zero), and `General` beside literals or in a date section (`[$-1B000409]General "x"` on 105, `[$-1B000409]d General` on 5.5). Write one probe case (excel-probe skill), fill it with excel-graph, and fix whatever differs.

## Acceptance criteria
- [ ] A probe case covers the four forms with oracle values; the code matches them, or the differences are recorded and filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

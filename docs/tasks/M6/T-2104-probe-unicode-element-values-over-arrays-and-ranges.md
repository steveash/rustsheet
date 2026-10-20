---
id: T-2104
title: "Probe UNICODE element values over arrays and ranges"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for UNICODE's element values when it lifts over an array or a range.

## Context
T-2090 made UNICODE lift like CODE (`fn_unicode`, `unicode_element` in functions/text/common.rs). Only the spill shape
is evidenced (gen-m9/675763979c1defb6 Calc!A25); the element values are hand-written expectations
(`fn_unicode.rs` `unicode_lifts_over_arrays_and_ranges`; text.md "UNICODE lifts over arrays and ranges", UNVERIFIED).
Probe: `UNICODE({"A","";TRUE,1.5})`, `UNICODE(A1:A3)` with A3 blank, a lone high and a lone low surrogate inside an
array (`UNICODE(MID("😀",{1;2},1))`), an error element (`UNICODE({"a";#N/A})`), and a LAMBDA argument.

## Acceptance criteria
- [ ] An oracle-filled probe case covers the cells above; the text.md entry is VERIFIED or corrected, with a
  follow-up task for any engine mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up does).

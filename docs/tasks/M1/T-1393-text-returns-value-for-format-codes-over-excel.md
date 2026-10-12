---
id: T-1393
title: "TEXT returns #VALUE! for format codes over Excel's complexity limit"
milestone: M1
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TEXT returns #VALUE! for format codes beyond Excel's complexity limit, matching the measured boundaries.

## Context
T-1388 found that Excel rejects far shorter format codes than 255 characters (16 placeholder/literal pairs ok, 17 fail; ~82 unquoted literal characters ok, 84 fail). See docs/excel-semantics/number-formats.md "TEXT format code complexity limit". The model (element weights) is unknown: needs more probing or fitting, hence opus. The engine only caps results at 255 (`format_number`).

## Acceptance criteria
- [ ] The probe cases text-date-255-char-limit, text-format-length-limit and text-format-length-limit-b match, or the remaining diffs are recorded as DEVIATION/unprobed
- [ ] `make check` green; reviewers approve.

## Out of scope
-

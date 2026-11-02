---
id: T-3598
title: "Second number run after a decimal point and literal prints no grouping"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`TEXT(1234567.89,"0.00|#,##0")` prints "1234567.89|0" as Excel does.

## Context
Found by T-1925: probes/t1925-numeral-digits C1, C2, C21-C27 (we print "1,234,567.89|0"). Semantics: number-formats.md "Unrelated quirk the probe found". Probe the other forms (`#,##0` before the point, other literals) before generalizing.

Cells (gap triage 2026-11-02): the same `0.00|#,##0` code under a numeral tag shows the same quirk (Excel prints no
grouping with every numeral byte, e.g. C3 `[$-2000409]` is "١٢٣٤٥٦٧.٨٩|٠", ours "١,٢٣٤,٥٦٧.٨٩|٠"), so the fix covers them:
- probes/t1925-numeral-digits.case.json (28): S!C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19, C20, C22, C23, C24, C25, C26, C28, C29, C30, C31, C32

## Acceptance criteria
- [ ] C1 and C2 pass, and so do the 28 numeral-tag cells listed above; a probe covers the neighbouring forms.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

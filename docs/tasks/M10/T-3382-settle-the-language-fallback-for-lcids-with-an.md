---
id: T-3382
title: "Settle the language fallback for LCIDs with an unknown sublanguage (0x2412)"
milestone: M10
lane: functions-text
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
---

## Goal
`[$-2412]` and other plain hex LCIDs with an invalid sublanguage print what Excel prints (English for 0x2412), not the language table.

## Context
T-1875 found `[$-2412]` prints English in Excel but `by_language` in formatter/locale_tag.rs gives Korean (probes/text-locale-regional-lcids-2 S!M1:M21 fail; docs/excel-semantics/number-formats.md "More regional variants"). Probe more sublanguage-invalid LCIDs (0x0812, 0x1412, 0x2411, 0x2407, 0xF409) to find the rule, then fix `by_language`.

## Acceptance criteria
- [ ] A probe case covers the LCIDs; the rule is in number-formats.md; text-locale-regional-lcids-2 M1:M21 pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

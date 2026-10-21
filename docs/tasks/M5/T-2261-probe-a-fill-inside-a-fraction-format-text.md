---
id: T-2261
title: "Probe a * fill inside a fraction format (TEXT and displayed text)"
milestone: M5
lane: functions-text
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A `*x` fill inside a fraction format matches Excel (TEXT and displayed text), by oracle evidence.

## Context
T-1735 made a `*x` fill print nothing in number and date sections. `formatter/fraction.rs` (`TextToken::Ghost(_) | TextToken::Spacer(_) => text.push(' ')`) still prints a space for a fill; no suite has a fraction code with `*`, so it is unprobed. Probe with `excel-probe`: `TEXT(0.5,"# ?/?*-")`, `TEXT(1.5,"*-# ?/?")`, `TEXT(1.5,"# ?*x/?")`, `TEXT(0,"# ?/?*-")` (blank-fraction path), then align fraction.rs and note it in number-formats.md "Fraction formats".

## Acceptance criteria
- [ ] Probe cases recorded with oracle values; fraction.rs matches them; number-formats.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

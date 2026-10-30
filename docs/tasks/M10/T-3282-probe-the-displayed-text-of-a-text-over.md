---
id: T-3282
title: "Probe the displayed text of a text over 255 characters under a text section"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A cell's displayed text for a text over 255 characters under a text-section format (`"pre "@`, `_(@_)`) matches Excel, from oracle evidence.

## Context
T-1736 routes a text or boolean cell through the format's text section. TEXT() is `#VALUE!` past 255 characters, and `formatter::format::displayed_text` keeps the plain text then; that is a guess, not Excel evidence (number-formats.md "Displayed text"). Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe records Excel's displayed text for 256- and 300-character texts under `"pre "@`, `@`, `_(@_)` and `;;;`.
- [ ] `displayed_text` matches it, or a `DEVIATION` records why not.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

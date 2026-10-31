---
id: T-1130
title: "Probe TEXT on text with 1-3 section formats, _x/*x and digit/date tokens in the text section"
milestone: M10
lane: functions-text
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence, and the engine matching it, for TEXT on a text or boolean value with formats T-0935 left unprobed.

## Context
T-0935 made `fn_text` apply the 4th (text) section, or a lone section containing `@`, to text and booleans (`format_text_value`, formatter/format.rs). docs/excel-semantics/number-formats.md ("TEXT of a text value is `#VALUE!` over 255 characters") lists what is unprobed and what the engine does today:
- a text value with a 1-3 section format and no lone `@` section, such as `TEXT("x","0.0")` or `TEXT("x","0;-0")`: the engine returns the text unchanged;
- `0;@`-style formats where a 2nd or 3rd section has `@`;
- `_x` and `*x` in the text section: the engine prints one space;
- digit or date tokens in the text section (`;;;0@`, `;;;d@`): the engine drops them;
- a General 4th section (`0;0;0;General`).
Use the `excel-probe` skill.

## Acceptance criteria
- [ ] Probe cells with oracle values for each bullet; the number-formats.md Unprobed line is resolved; the engine matches (baseline shrinks or a DEVIATION entry).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

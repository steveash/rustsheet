---
id: T-3746
title: "NUMBERVALUE with NBSP or U+3000 as a separator at the start or end of the text"
milestone: M5
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-3647]
---

## Goal
NUMBERVALUE gives Excel's value when NBSP (CHAR(160)) or U+3000 is the decimal or group separator and stands at the
start or end of the text.

## Context
- Deferred MINOR of the excel-fidelity review of PR #2754 (T-3647). T-3647 found NBSP and U+3000 at the ends of the
  text are `#VALUE!` (probes/t3647-numbervalue-whitespace-decimal A25-A27) and refuses them before the decimal
  separator is replaced (`numbervalue_of` in functions/text/string_format.rs). So with NBSP as the decimal separator,
  `NUMBERVALUE("1"&CHAR(160),CHAR(160))` and `NUMBERVALUE(CHAR(160)&"5",CHAR(160))` are now `#VALUE!` (before T-3647:
  1 and 0.5). No probe has NBSP or U+3000 as a separator.
- Probe with the `excel-probe` skill: NBSP and U+3000 as the decimal separator and as the group separator, each at the
  start, the end and inside the text; then match.

## Acceptance criteria
- [ ] A probe case (excel-graph) covers those cells; the engine matches every one; text.md "NUMBERVALUE removes group
      separators only before the decimal separator" states the rule.
- [ ] `make check` green; eval known failures only shrink (baseline updated); reviewers approve.

## Out of scope
- VALUE (T-3742).

---
id: T-3342
title: "TEXT: probe mixed-case General spellings and a comma before General"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence confirms how `TEXT` reads mixed-case `General` spellings and a comma before `General`, and
number-formats.md states the rule.

## Context
T-1614 probed `General` company (probes/text-format-general-company; docs/excel-semantics/number-formats.md).
- The lexer matches `General` in any mix of case (`Lexer::looks_at`), but only the all-lowercase and all-uppercase
  spellings were probed.
- A comma before `General` is unprobed.

Probe (excel-probe skill):
- `"gEnErAl x"` and `"GeNeRaL"` on 5.25, 0 and -1;
- a leading or middle comma: `",General"` and `"#,General"`.

`General` after a date or time token (`TEXT(5,"d General")` = "5 0.5", A5 of that case, plus `"m General"`,
`"General d"` and `"yyyy General"`) is T-3378's, so it is not part of this task.

## Acceptance criteria
- [ ] A probe case covers the formats above (excel-graph) and is recorded. number-formats.md states the rule for each,
      marked VERIFIED, and any cell the engine fails is a known failure owned by a filed follow-up.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `General` inside a date or time section: T-3378.

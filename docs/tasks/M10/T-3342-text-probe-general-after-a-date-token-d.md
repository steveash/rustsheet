---
id: T-3342
title: "TEXT: probe General after a date token (d General = 5 0.5) and mixed-case General"
milestone: M10
lane: functions-text
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know and match why Excel gives `TEXT(5,"d General")` = "5 0.5", and confirm mixed-case `General` spellings.

## Context
T-1614 probed `General` company (probes/text-format-general-company; docs/excel-semantics/number-formats.md). The engine
says `#VALUE!` for A5 `TEXT(5,"d General")`, Excel "5 0.5" (known failure in eval/baseline/probes.txt). The lexer matches
`General` with any case mix (`Lexer::looks_at`), but only all-lowercase and all-uppercase were probed. Probe (excel-probe
skill): `"d General"` on 5.25 and 0, `"m General"`, `"General d"`, `"yyyy General"`, `"gEnErAl x"`, and a leading or middle
comma (`",General"`, `"#,General"`).

## Acceptance criteria
- [ ] A probe case covers the formats above (excel-graph) and passes or is recorded; number-formats.md states the rule (VERIFIED); A5 implemented if the rule is clear.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

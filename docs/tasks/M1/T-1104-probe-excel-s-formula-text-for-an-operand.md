---
id: T-1104
title: "Probe Excel's formula text for an @ operand of :, # and @"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how Excel prints (FORMULATEXT) an `@` that is an operand of `:`, `#` or `@`: `A1:(@B1)`, `(@(A1 B1)):C3`, `(@A1:B2)#`, `@(@(A1 B1))`.

## Context
T-1102 parenthesizes these operands (`tight_operand`, `at_operand` in crates/rustsheet-engine/src/expressions/parser/reference_operators.rs) as the minimal engine choice so the text re-parses; Excel's own text is unprobed. Use the `excel-probe` skill (FORMULATEXT cells). Also probe `=ANCHORARRAY(name)` and `ANCHORARRAY(INDEX(..))`: xlsx import adds an automatic `@` inside `#` (static_analysis.rs, `add_implicit_intersection`), which now prints `(@name)#`; Excel probably shows `name#`, and the automatic `@` may need dropping there.

## Acceptance criteria
- [ ] A probe case holds oracle FORMULATEXT values for each form; docs/excel-semantics/ records them; an engine task is filed if the printed text differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

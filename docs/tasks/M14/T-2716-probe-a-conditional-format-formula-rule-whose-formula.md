---
id: T-2716
title: "Probe: a conditional format formula rule whose formula returns an array"
milestone: M14
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for which cells a conditional-format "use a formula" rule formats when its formula
returns an array (`IF(J1:J10,"open")`, `J1:J3>0`, `{TRUE,FALSE}`), recorded in
docs/excel-semantics/ and matched by the engine.

## Context
`apply_cf_formula` (conditional_formatting.rs) treats any array result as no match (`_ => false`),
with no evidence. Excel may take the top-left element, or apply implicit intersection. Real files
carry such rules: corpus-bench 41978 (T-0430, T-0468). Formatting is not a cell value, so the
probe needs a way to read applied formats (desktop Excel, `DisplayFormat`), hence M14.

## Acceptance criteria
- [ ] A probe workbook and its recorded observations for the forms above, cited in
      docs/excel-semantics/.
- [ ] The engine matches, or a follow-up task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making conditional formats evaluate on demand (T-0468).

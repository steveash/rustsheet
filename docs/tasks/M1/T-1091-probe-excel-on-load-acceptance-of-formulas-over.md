---
id: T-1091
title: "Probe Excel on-load acceptance of formulas over 8,192 characters (chains past 4,096 operators)"
milestone: M1
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for whether Excel loads and evaluates `=1+1+...` chains longer than 8,192
characters, and what happens at the real on-load limit.

## Context
T-1060 found Excel for the web evaluating an 8,193-character formula loaded from a file, so the
published 8,192 limit is not enforced on load. RustSheet refuses chains past `MAX_VALUE_OPERATORS`
(4,096); a longer chain Excel accepts would be refused. See docs/excel-semantics/syntax.md
("Formula nesting depth cap"); code: crates/rustsheet-engine/src/expressions/parser/depth.rs.

## Acceptance criteria
- [ ] Probe case with `=1+1+...` chains of 8,196, 8,200, 16,384 and 32,767 characters, oracle-filled.
- [ ] syntax.md records the on-load limit (or that none was found) and, if Excel accepts more than
  4,096 operators, a follow-up task proposes the change (decision record needed: DEVIATION).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing `MAX_VALUE_OPERATORS` in this task.

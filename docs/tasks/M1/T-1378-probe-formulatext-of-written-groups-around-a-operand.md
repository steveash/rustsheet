---
id: T-1378
title: "Probe FORMULATEXT of written groups around a % operand and a LAMBDA callee: =(A1)%, =(LAMBDA(x,x))(1)"
milestone: M1
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for how Excel prints written groups in shapes no probe covers yet, so the engine's text for them is backed by evidence rather than by the general rule.

## Context
T-1340 (`Node::Group`, SPEC-0007) prints every written group as written, by the verified rule in docs/excel-semantics/syntax.md "FORMULATEXT keeps every written parenthesis". Unit tests now pin texts for unprobed shapes: `=(A1)%`, `=(LAMBDA(x,x))(1)`, `=(1/2)/3`, `=1+(3+5)`, `=(D:D 2:2)&""`, `=(-A1)^2` (test_grouped_root.rs, test_general.rs, test_stringify.rs, test_intersection.rs, test_group.rs). Follow the excel-probe skill.

## Acceptance criteria
- [ ] A probe case (excel-graph) with the formulas above and FORMULATEXT of each, oracle filled and baseline recorded.
- [ ] Any shape where Excel's text differs from the engine's gets an excel-semantics entry and a follow-up engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

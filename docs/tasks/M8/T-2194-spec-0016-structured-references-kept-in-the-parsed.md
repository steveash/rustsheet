---
id: T-2194
title: "SPEC-0016: structured references kept in the parsed formula"
milestone: M8
lane: engine
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A reviewed spec says how structured references stay in the parsed formula: kept in storage and export, re-resolved when the table list changes, and `#This Row` decided at evaluation. With it, T-1699 and T-1683 can be implemented.

## Context
T-1699 found that the parser lowers every structured reference to a range at parse time (`structured_reference_operand`, parser/operands.rs). `to_rc_format` then stores that range, so `=SUM(Sales[Qty])` is stored and exported as `=SUM($A$2:$A$3)`, and `=Sales[@Qty]` off the data rows as `=#VALUE!`. Re-adding the table over a larger range changes neither. Shared formulas are parsed with the context (1, 1), so a preserved `#This Row` cannot be resolved at parse time. Fixing this changes the evaluation model, so it needs a spec first (CLAUDE.md directive 4).

## Acceptance criteria
- [ ] `docs/specs/SPEC-0016-structured-references-in-the-parsed-formula.md` covers the node design, transparency (`ungrouped()`), printers, shared formulas, alternatives, test plan and rollout.
- [ ] Rollout tasks filed: T-2195 (step 1), and T-1699 re-scoped as step 2 with `depends_on: [T-2195]`.
- [ ] `make check` green; the architecture reviewer approves.

## Out of scope
- Implementation (T-2195, T-1699).

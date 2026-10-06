---
id: T-1030
title: "Reconcile CLAUDE.md directive 1 and decision 0014 with non-directive-5 deviation records"
milestone: M1
lane: quality
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The governing documents agree on when RustSheet may deviate from Excel outside directive 5.

## Context
Raised by the Excel-fidelity reviewer on T-1025. CLAUDE.md directive 1 and decision 0014 say
directive 5 is the only exception to "Excel is always right", while docs/excel-semantics/README.md
("Deliberate deviations") allows any other deviation through its own decision record, and decisions
0033 (lone surrogates) and 0038 (WEEKNUM calc-order state) use that route. CLAUDE.md is a protected
path: the change needs its own decision record approved by reviewers.

## Acceptance criteria
- [ ] CLAUDE.md directive 1, decision 0014 and the semantics README state the same rule (either allow per-decision deviations, with what qualifies, or forbid them and reopen 0033/0038).
- [ ] Decision 0036's `deviations` file scope is stated for non-directive-5 deviations.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

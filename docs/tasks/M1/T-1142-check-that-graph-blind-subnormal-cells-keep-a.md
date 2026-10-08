---
id: T-1142
title: "Check that graph-blind subnormal cells keep a (x)=0 companion (decision 0042)"
milestone: M1
lane: harness
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A cell that decision 0042 scores as a pass (an excel-graph 0 against a kept subnormal) is reported when its case has no `(x)=0` companion cell that evidences the flush-or-keep result. A wrongly kept subnormal therefore cannot pass unseen.

## Context
`graph_hides_subnormal` (crates/rustsheet-eval/src/runner.rs, T-1027) scores such a cell as a pass because Graph reports a subnormal as 0. Decision 0042 relies on companions (`=(<formula>)=0`), as in probes/subnormal-function-results C-column. Nothing enforces that a companion exists. Suggested approach: in the report, list the cells the rule passed. For a spec case, flag one when no cell in the case has the formula `=(<its formula>)=0`.

## Acceptance criteria
- [ ] The report (or eval-check, as a warning) lists the cells decision 0042 passed and flags any without a companion; unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the scoring rule.

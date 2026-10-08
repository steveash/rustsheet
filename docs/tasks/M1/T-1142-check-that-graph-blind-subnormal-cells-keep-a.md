---
id: T-1142
title: "Check that graph-blind subnormal cells keep a (x)=0 companion (decision 0042)"
milestone: M1
lane: harness
status: done
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

## Outcome
- Decision 0042's scoring already required a FALSE `(<formula>)=0` excel-graph companion (`from_graph` in `prepare`), so a cell without one is never passed by the rule; it scores as a mismatch.
- Added visibility: `CellResult.graph_subnormal` marks each cell the rule passed. `SuiteSummary.graph_subnormal_passes` lists them; the markdown report and the CLI summary show them. Unit test in `report.rs`.
- Eval delta: none (scoring unchanged).

---
id: T-3251
title: "Typed minus-dollar mixed fraction with percent must stay text, not #ERROR!"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Typed `-$3 1/2%` stays a text cell, as in Excel, instead of evaluating to `#ERROR!`.

## Context
- Owns the M2 exit cells (decision 0051: the case is tagged M2, so it counts in the M0-M2 probe row;
  moved from M9/functions-text to M2/engine by the planner on 2026-10-31, when `make exit-owners
  MILESTONE=M2` listed B15 as having no open M2 owner; the fix is in the engine's typed-input path,
  `parse_typed_number` in crates/rustsheet-engine/src/formatter/parse_formatted.rs and
  `mixed_fraction` in crates/rustsheet-engine/src/mixed_fraction.rs):
  - probes/typed-mixed-fraction-number-format.case.json (2): Sheet1!A15, B15
- On main at fbcd47e9 (`make eval-show CASE=probes/typed-mixed-fraction-number-format.case.json`):
  B15 `=A15+0` is 0 (Excel `#VALUE!`) and A15's displayed text is empty (Excel `-$3 1/2%`), so the
  typed text is lost rather than shown as `#ERROR!` as first observed below; find where it goes.
- First observation (T-3237): probes/typed-mixed-fraction-number-format A15/B15 (excel-graph): Excel keeps the text `-$3 1/2%` (`=A15+0` is `#VALUE!`); ours shows `#ERROR!`, so the typed text starting with `-` seems to reach the formula path. Found by T-3237; `mixed_fraction` already rejects the input. Excel still gives that cell the number format `0.00%` (A15 ENTERED numberFormat), so the fix sets the format too.

## Acceptance criteria
- [ ] A15/B15 of that probe pass (TF/F lines leave eval/baseline/probes.txt).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

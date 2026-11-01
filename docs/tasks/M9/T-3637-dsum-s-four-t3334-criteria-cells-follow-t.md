---
id: T-3637
title: "DSUM's four t3334 criteria cells follow T-3404's 1e2$ rule (M9 owner)"
milestone: M9
lane: functions-text
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3404]
---

## Goal
DSUM over the criteria operands of probes/t3334-criteria-operand-forms gives Excel's results, so DSUM's four
attributed cells there have an M9 owner and pass.

## Context
- probes/t3334-criteria-operand-forms Calc!G39, G40, G41 and G42: `DSUM(Data!$F$1:$G$15,"w",DC!A77:A78)` and the
  next three, with criteria `1e2$`, `=1e2$`, `<1e2$` and `>1e2$`. Excel reads `1e2$` as text, so the plain and `=`
  rows match nothing (G39 is 0, ours 32).
- T-3404 (M2) implements that rule for every criteria consumer, and its Context names Calc!A39:G42. These four are
  DSUM's, an M9 function, and they sit in M9's per-function rate criterion. An M2 task cannot own them (decision
  0096), so this task does. Once T-3404 lands, confirm they pass and close it.
- Found by `make exit-owners MILESTONE=M9` on 2026-11-01.

## Acceptance criteria
- [ ] probes/t3334-criteria-operand-forms Calc!G39:G42 pass (`make eval-baseline` drops them).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The non-DSUM cells of rows 39 to 42 (T-3404).

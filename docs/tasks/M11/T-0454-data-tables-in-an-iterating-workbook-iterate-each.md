---
id: T-0454
title: "Data tables in an iterating workbook iterate each substitution's cycles"
milestone: M11
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-0447]
---

## Goal
In a workbook with iterative calculation on, each data-table substitution gives the result Excel
gives. Calipers pv_goldens/pv26_014 (and pv26_2_014) Sensitivity!E39:H39 match Excel.

## Context
After T-0441 and T-0447, a data table's substituted evaluation (`evaluate_substituted` in
crates/rustsheet-engine/src/data_tables.rs) reads a cycle's current values. It does not sweep the
cycle with the input replaced. In pv26_014 four results differ from Excel by up to 1e-3 relative,
for example E39 0.45576539857680487 against Excel's 0.4563532962983179. Excel most likely iterates
each substitution (recalc.md, "Iterative calculation" Ours line).

Probe it first (excel-probe skill), with a one-variable table whose formula reads a convergent
cycle, iteration on. Record the rule in recalc.md. Then, when it holds, run the members' sweeps
(`iteration.rs`) inside each substitution, within the same work budgets.

## Acceptance criteria
- [ ] A probe case with Excel values covers a data table over a cycle; recalc.md records the rule.
- [ ] pv26_014 and pv26_2_014 Sensitivity!E39:H39 leave eval/baseline/calipers.txt
      (`rustsheet-eval update-baseline --suite calipers`), with no new known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR precision in the same workbooks (T-2131).

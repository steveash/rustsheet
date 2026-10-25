---
id: T-0464
title: "Probe circular cells with no cached value, a spilling anchor on a loop and a cached text or error value (T-0461 follow-up)"
milestone: M11
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the three cases of the circular-reference rule (docs/excel-semantics/recalc.md, "A circular
reference without iteration keeps the cells' values from before the recalculation") that T-0461 left at 0 with no
oracle cell behind them.

## Context
T-0461 found that, with iteration off, the cells of a cycle and the cells that read one keep their pre-recalculation
value. Before that the rule was "is 0", but every earlier probe was built by spec2xlsx through xlsxwriter, which caches
every formula as 0, so those probes could not tell "is 0" from "keeps 0". The engine (`Model::circular_value`,
crates/rustsheet-engine/src/circular.rs) still gives 0 in three cases with no evidence:
- a formula on a loop with no cached `<v>`: the xlsx import makes it `FormulaValue::Unevaluated`
  (crates/rustsheet-xlsx/src/import/worksheets.rs), and it is 0;
- a multi-cell spilling anchor on a loop whose earlier spill is cached: 0, with no spill;
- a newly entered formula that closes a loop (unit test `test_circular_references.rs`, "B1's formula was never
  evaluated: 0"), a hand-written expectation.

Also unprobed: a loop member whose cached value is text, a boolean or an error. The rule says it keeps it; only
numbers are evidenced. And a driver-marked array anchor (`circular_anchors`) stored as a one-cell array with a cached
value now keeps that value; recalc.md says so, unprobed.

## Acceptance criteria
- [ ] A fixture (hand-built xlsx, like fixtures/circular-keeps-cached-values, so the cached `<v>` values are chosen)
      with: a loop member with no `<v>`; a multi-cell dynamic anchor on a loop with cached spill values; loop members
      cached as text, TRUE and `#DIV/0!`. Oracle-filled by excel-graph (canary-verified).
- [ ] recalc.md records each result; the engine follows, or a task is filed for each difference.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iterative calculation (iteration on).

---
id: T-0445
title: "A circular reference without iteration is 0, not #CIRC!, for its cells and readers"
milestone: M3
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
With iterative calculation off, every cell of a cycle, and every cell that reads one, is the number 0
with no error value, as in Excel; ours makes each of them `#CIRC!`.

## Context
Filed by T-1965. docs/excel-semantics/recalc.md "A circular reference without iteration is 0, and so
is every cell that reads it" (probes/circular-iteration-off, excel-graph, T-0428) states the rule;
its "Ours" line says every cycle cell and reader is an error. Known failures that are owned here:
probes/circular-iteration-off (10 cells in eval/baseline/probes.txt) and probes/vstack-self-past-grid
W9 (`=ROWS(VSTACK(A:A,B9+W9))` in W9 is 0 in Excel; T-1965 found it). Do not change what IF and
ROW/COLUMN already do: neither reads the cell (circular-iteration-off), and neither do ROWS,
COLUMNS and ISREF of the formula's own row (vstack-self-past-grid W5-W7). Excel-graph's 0 for
probes/scalar-lift-open-c3 W1 (a 2^34-cell HSTACK) is not owned here: it is past the directive-5
cell-count bound (arrays.md "Stacked results past the grid"). Cycle detection is in the evaluator
(`#CIRC!` paths in crates/rustsheet-engine/src/model.rs); T-0401 (SPEC-0002, the dependency graph)
may move it, so check that task's state first and coordinate rather than duplicate.

## Acceptance criteria
- [ ] The probes/circular-iteration-off lines and probes/vstack-self-past-grid W9 leave
      eval/baseline/probes.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] recalc.md's "Ours" line states the new behavior.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iterative calculation (recalc.md's other sections).
- The 2^34-cell HSTACK result (-c3 W1).

---
id: T-0451
title: "Probe circular references without iteration: a spilling reader, and readers that take a reference without its value"
milestone: M3
lane: research
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel's values are known for two kinds of readers of a circular reference (iteration off) that
probes/circular-iteration-off and probes/t0445-circular-readers leave open.

## Context
Filed by T-0445 (Excel-fidelity review). recalc.md "A circular reference without iteration is 0"
says that a loop and every reader of it are 0. Two kinds of reader are unprobed:
- A formula that would spill while it reads a loop, for example `=SEQUENCE(3)+A1` with `A1` `=A1+1`.
  Is it one 0, or a spilled array? Ours stores a single 0 and does not spill.
- Readers that take a reference without reading the value behind it, pointed at a loop cell:
  `ISREF(A1)`, `CELL("row",A1)`, `ROWS(A1)`, `COLUMNS(A1:B1)`, `OFFSET(A1,1,0)` (where the
  target cell holds a constant), and `INDIRECT("A1")`.
- The control is ROW/COLUMN of a formula's own cell, which is not a read (circular-iteration-off D2).
Write one probe case under eval/suites/probes (tag M3), fill it on excel-graph, and record the
answers in recalc.md. If ours differs, file a functions/engine task for the cells that are off.

## Acceptance criteria
- [ ] probes/<new case> is filled on excel-graph, with the cells above.
- [ ] recalc.md states the answers, and any cell where ours differs has its own task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Iterative calculation.

## Outcome
Added probes/t0451-circular-spill-and-ref-readers (filled on excel-graph). Excel: a would-be spill that reads a
loop is one 0 with no spill; reference-only readers (ISREF, CELL, ROWS, COLUMNS, OFFSET, INDEX, ISFORMULA) are
not reads, INDIRECT is. Ours already matches on all 17 cells, so no follow-up task. Eval delta: +1 case, +17 cells,
all bit-exact; no baseline change.

---
id: T-1562
title: "Cell lines for the later-milestone cells of mixed probe cases counted for M0-M2"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A probe case that keeps its M0-M2 tag for an M2 subject has no checked cell whose only subject is a later-milestone function without a cell line in eval/milestones/probe-milestones.tsv, so a later regression in such a cell never weighs on M2 exit.

## Context
Decision 0059 (T-1547) mapped cases whose subjects are all later-milestone and gave cell lines to the mixed cases it flagged (XOR, SWITCH, CHOOSE, `multiarea-more`). Its fidelity review found kept mixed cases it did not sweep, for example `legacy-value-argument-intersection-b` (about 30 cells calling MINA, STDEV.S, GEOMEAN … beside MIN, SUM, CONCAT). None fails today. Also settle `rich-errors-calc` A1, A4 (blocked SEQUENCE `#SPILL!`, FILTER `#CALC!`) and their B/C observers: 0059 keeps the case, while the same cells in `rich-errors-image` (A3, A6) went to M7 (spill semantics are M7 scope in docs/milestones.md). Sweep per cell: every checked cell of an M0-M2 case whose non-observer catalog functions are all above M2 (0051 item 2, 0058's argument-builder rule, 0059's two readings). eval/milestones/ is protected: amend by a decision record.

## Acceptance criteria
- [ ] A decision record lists the new cell lines (or the reason a cell keeps its case's milestone); `make eval-check` passes and the record gives the M0-M2 counts before and after.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing any cell; editing case.json tags (decision 0043).

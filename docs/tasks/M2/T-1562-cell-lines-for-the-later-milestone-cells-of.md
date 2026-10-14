---
id: T-1562
title: "Cell lines for the later-milestone cells of mixed probe cases counted for M0-M2"
milestone: M2
lane: research
status: done
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
- [x] A decision record lists the new cell lines (or the reason a cell keeps its case's milestone); `make eval-check` passes and the record gives the M0-M2 counts before and after.
- [x] `make check` green; reviewers approve.

## Out of scope
- Fixing any cell; editing case.json tags (decision 0043).

## Outcome
Decision 0060 adds 4 case lines and 1,086 cell lines to eval/milestones/probe-milestones.tsv.
- The per-cell sweep (every checked cell of a case counted for M0-M2 whose non-observer
  functions are all above M2) flagged 2,020 cells in 117 cases.
- 1,216 cells move to their reader's milestone: M5 594, M6 292, M7 23, M8 6, M9 267 and M10 34,
  counting the case lines. That is the 1,208 flagged cells that move, plus 8 unflagged ones: the
  `rich-errors-calc` observers and the rest of `external-ref-no-link-ifchoose-b`.
- 812 keep the case's milestone under five named rules: coercion grammar, an operator counted or
  built, a reference construct built by a later function, data cells, and cases already decided
  by name.
- `rich-errors-calc` A1 and A4 (SEQUENCE `#SPILL!`, FILTER `#CALC!`) and their B/C observers go
  to M7, matching `rich-errors-image`.
- `make eval` probes M0-M2: 292 cases / 9,328 cells / 177 known failures before, and
  288 / 8,112 / 171 after. Only the six `trig-huge-argument` failures moved (to M5), and T-1563
  moved to M5 with them. `make eval-check` shows 0 regressions, and no baseline changes.

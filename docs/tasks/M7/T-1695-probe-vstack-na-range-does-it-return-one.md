---
id: T-1695
title: "Probe VSTACK(NA(),range): does it return one #N/A instead of an array?"
milestone: M7
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Know whether Excel's VSTACK with a scalar error as its first argument, e.g. `VSTACK(NA(),{1;2})`, returns one `#N/A` instead of a 3-row array, and record it.

## Context
T-1323's probe groupby-sort-filter-args2 question B36 suggested `VSTACK(NA(),range)` yields a single `#N/A` (the whole summary cell was `#N/A`), but only that summary cell was probed. The engine's VSTACK gives the same answer today. The T-1323 fidelity review (PR #916) asked for a direct probe. See docs/excel-semantics/arrays.md, GROUPBY section, "Review follow-up".

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) answers `ROWS(VSTACK(NA(),{1;2}))`, `ISERROR(VSTACK(NA(),{1;2}))`, `INDEX(VSTACK(NA(),{1;2}),2)`, and the same with HSTACK and with an error in a later argument.
- [ ] The finding is recorded under the VSTACK/HSTACK entry in docs/excel-semantics/arrays.md; if the engine differs, a fix task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing VSTACK/HSTACK unless the probe shows a mismatch (file a task).

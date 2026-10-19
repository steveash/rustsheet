---
id: T-1484
title: "Spec: WEEKNUM stale return_type state in Excel's calc order (decision 0038 reopened)"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A spec (docs/specs/) for how RustSheet reproduces WEEKNUM's stale return_type state, which
follows Excel's calc order, and the tasks to implement it.

## Context
Decision 0052 reopened decision 0038: the stale state is a known gap, not a deviation. In Excel,
`WEEKNUM(serial, return_type)` with a return_type of 22 or more, or negative, reuses the start
weekday of the last valid WEEKNUM evaluated earlier in the same recalc, and is `#NUM!` only when
none ran (docs/excel-semantics/dates.md "WEEKNUM return_type edge cases and serial 0").
probes/weeknum-stale-return-type Up!A1, Mixed!A3, Mixed!A4, Type21!A1 and Type1!A1 are known
failures; `fn_weeknum` always gives `#NUM!`. On a fresh load the probes fit "bottom to top within
a sheet"; after edits Excel follows its calc chain (`calcChain.xml`). Decision 0038's options (a)
and (b) list the problems: RustSheet's storage-order passes, restarts on spill reordering, and a
hidden dependency that incremental recalc cannot see.

## Acceptance criteria
- [ ] Probe what decides the order across sheets and after edits (excel-probe skill), and record it
      in docs/excel-semantics/dates.md.
- [ ] docs/specs/SPEC-NNNN-*.md states how the state is kept and which calc order it follows, and
      files implementation tasks with `make new-task`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing calc-chain import in general, beyond what the spec needs.

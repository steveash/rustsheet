---
id: T-2070
title: "Probe WEEKNUM's stale state: array readers, spill anchors, names, LET/LAMBDA, conditional formats"
milestone: M6
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the cases SPEC-0015 leaves open about WEEKNUM's stale return_type state.

## Context
docs/excel-semantics/dates.md "WEEKNUM's stale return_type state" lists these as unprobed. Use the
marker and reader technique of probes/weeknum-stale-setters: A1 reader, A2 the call under test, A3 a
marker; the order is bottom to top. Write one probe case per question group and tag it M6.

## Acceptance criteria
- [ ] Probe cases, filled by excel-graph, for:
  - a reader that lifts over an array, `=SUM(IFERROR(WEEKNUM(100,{99,99}),0))`, after a Monday marker;
  - a spilling formula: a reader spilling (`=WEEKNUM(100,99)+SEQUENCE(2)*0`) above and below a
    marker, to show where anchors sit in the order;
  - WEEKNUM inside a defined name, inside LET and inside a LAMBDA called from a cell, as both setter
    and reader;
  - a reader in a conditional-formatting rule, if the harness can read it; otherwise say it cannot;
  - return_type -0.5 and 21.5, and serial 2958466 (`#NUM!`) with type 2.
- [ ] dates.md updated with each finding. Any finding that changes SPEC-0015's design becomes a
      follow-up engine task (`make new-task`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementation (T-2068, T-2069).

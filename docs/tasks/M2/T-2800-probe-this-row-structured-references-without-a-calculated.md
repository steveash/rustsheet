---
id: T-2800
title: "Probe this-row structured references without a calculated-column fill"
milestone: M2
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for whitespace in this-row structured references.

## Context
T-2786 could not answer: typing a this-row formula into a table column made Excel fill the whole column (calculated column), overwriting the other probes. Probe `=T[[#This Row], [Col]]`, `=T[ [#This Row] , [Col] ]` and `[@ Col]`-style shapes in a table column where each is entered alone (separate cases, or outside the table via a one-row table), with FORMULATEXT.

## Acceptance criteria
- [ ] A stored (file) `<f>SUM(T[ Col ])</f>` case answers whether Excel loads it (syntax.md leaves it unprobed).
- [ ] Probe case with excel-graph values; findings in syntax.md; implementation task filed if it differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing the findings.

## Outcome
Probed with excel-graph (probes/t2800-this-row-whitespace, t2800-stored-ws-0..3). Stored `T[ Col ]`, `T[ Col]`, `T[Col ]`
load as `#REF!`; stored `T[ [Col] ]` loads (prints `T[ Col ]`); stored `T[ #Data ]` corrupts the workbook. This-row shapes
print `[@Col]` / `[ @Col ]` inside a table. Findings in syntax.md; implementation filed as T-3069. Eval: five new probe cases,
three failing for us (not in the baseline), no regressions.

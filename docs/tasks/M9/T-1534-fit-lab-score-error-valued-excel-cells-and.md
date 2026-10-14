---
id: T-1534
title: "Fit lab: score error-valued Excel cells and per-iterate traces"
milestone: M9
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The fit lab can score a candidate on cells whose Excel value is an error, and can report which
iterate of a candidate's trace equals Excel's value.

## Context
T-1525's solver fits (`tools/src/rustsheet_tools/fitlab/kernels/solvers.py`) needed both. The lab
scores numeric Excel values only (`sweep.select` counts the rest as non-numeric), so the XIRR
`#NUM!` agreement (19/19 in the fit set, 8/11 out of sample) is a helper
(`solvers.xirr_error_agreement`) checked by `tests/test_fitlab_solvers.py`, not a lab column. Trace
analysis (is Excel's value iterate k of method M; the bisection depth per cell; the feasible bound
of a stop-test constant) was done in scratch scripts and is quoted in
`docs/excel-semantics/financial.md` as derived outside the lab.

## Acceptance criteria
- [ ] A candidate can return an error (e.g. `#NUM!`); error cells are scored (match, wrong error, number instead of error) and shown in the table.
- [ ] A candidate can expose its iterates; the report gives per cell the index of the iterate equal to Excel's value, if any.
- [ ] solvers.py uses both; the XIRR fit's table shows 227/227.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New solver models (T-1533).

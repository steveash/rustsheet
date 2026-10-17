---
id: T-1751
title: "MDETERM of a singular 3x3 matrix: reproduce Excel's 6.66e-16 residual"
milestone: M9
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
MDETERM(G1:I3) of the singular matrix [1,2,3;4,5,6;7,8,9] gives Excel's 6.661338147750939e-16 instead of 0.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q107 (excel-graph). Excel's elimination order leaves a rounding residual; ours gives exactly 0. Needs the pivoting/elimination order that reproduces it (numeric fitting, so opus).

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

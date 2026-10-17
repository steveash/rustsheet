---
id: T-1754
title: "KURT with a direct TRUE argument differs in the last digits"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
KURT with a direct TRUE argument matches Excel's last digit.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q36 (`KURT(A1:A8,TRUE)`): Excel -1.4331834319526635, ours -1.4331834319526626 (about 9e-16, roughly 4 ulp; likely the accumulation order). Numeric fitting, so opus.

## Acceptance criteria
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

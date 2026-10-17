---
id: T-1750
title: "STANDARDIZE of a range lifts element-wise (spills) instead of #VALUE!"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
STANDARDIZE of a range argument lifts element-wise like Excel (`=STANDARDIZE(A1:A2,3,2)` spills; it is #SPILL! only when blocked) instead of #VALUE!.

## Context
Found by T-1623: probes/stats-evidence-floor-t1623 S!Q63 (excel-graph; the spill was blocked by data below, giving #SPILL!, which proves Excel lifted). Ours gives #VALUE!. See the array-lift rules in docs/excel-semantics/.

## Acceptance criteria
- [ ] An unblocked probe (nothing in the spill range) records the spilled values of `STANDARDIZE(A1:A2,3,2)`.
- [ ] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [ ] The rule is recorded in docs/excel-semantics/.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

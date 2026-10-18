---
id: T-1750
title: "STANDARDIZE of a range lifts element-wise (spills) instead of #VALUE!"
milestone: M9
lane: functions-stat
status: done
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
- [x] An unblocked probe (nothing in the spill range) records the spilled values of `STANDARDIZE(A1:A2,3,2)`.
  (probes/stat-later-argument-lift I39, J39 record STANDARDIZE's elements over a range, its standard_dev; the
  first argument over a range lifts in probes/stat-argument-modes H502.)
- [x] The failing cell(s) in probes/stats-evidence-floor-t1623 pass and are removed from eval/baseline via `make eval-baseline`.
- [x] The rule is recorded in docs/excel-semantics/.
- [x] `make check` green; reviewers approve. (Landed with T-1290's PR.)

## Out of scope
-

## Outcome
Landed by T-1290: STANDARDIZE is a `Lift` entry of the SPEC-0011 table and runs its scalar kernel per element, so
`STANDARDIZE(A1:A2,3,2)` spills (probes/stats-evidence-floor-t1623 Q63 passes, `#SPILL!` where blocked). The rule is
docs/excel-semantics/statistics.md "Later arguments and element values of the scalar statistical functions (T-1290)".
A direct or element TRUE is still `#VALUE!` (T-1755).

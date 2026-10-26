---
id: T-2760
title: "Fit IMTAN/IMCOT real-argument kernel near pi/2 (probes/eng-imtan-imcot-overflow A10-A13)"
milestone: M9
lane: functions-engineering
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
IMTAN and IMCOT of a real argument at and near the double nearest pi/2 print Excel's digits.

## Context
T-1966 probed it: Excel gives IMTAN("1.5707963267948966") = 148752068975966, ours 148752623989354 (differs from the 6th or 7th digit), and likewise for the 1.5707963267949 and IMCOT cells. Failing cells: probes/eng-imtan-imcot-overflow.case.json A10-A13 (eval/baseline/probes.txt). Notes: docs/excel-semantics/engineering.md "IMTAN and IMCOT use the double-angle form"; code: `tan_parts` in crates/rustsheet-engine/src/functions/engineering/complex/trig.rs. Numeric fitting of the kernel near a pole.

## Acceptance criteria
- [ ] The four cells A10-A13 of probes/eng-imtan-imcot-overflow pass and leave eval/baseline/probes.txt (`make eval-baseline`), with no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

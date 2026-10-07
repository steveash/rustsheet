---
id: T-1076
title: "GEOMEAN of 1E+100 and 9E+307 is 1 ulp high in the overflow path"
milestone: M1
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
`=GEOMEAN(1E+100,9E+307)` equals Excel's 9.486832980505138e203 (the engine gives ...139, one ulp high).

## Context
Found by T-1032: probes/geomean-mixed-magnitude Sheet1!A1 (baselined). The plain product overflows, so `fn_geomean` (functions/statistical/geomean.rs) takes the power-of-two-scaled n-th root path; GEOMEAN(1E-100,1E-300) and GEOMEAN(1E+154,1E+300) already match. Excel's exact algorithm is unknown, so this is numeric fitting against oracle values (add more probe cells if needed).

## Acceptance criteria
- [ ] Sheet1!A1 of probes/geomean-mixed-magnitude passes and leaves eval/baseline/probes.txt, with no regression of the other GEOMEAN cases (numbers.md, ironcalc-calc GEOMEAN).
- [ ] `make check` green; reviewers approve.

## Out of scope
-

---
id: T-2931
title: "GAUSS(1E-15) is one ulp below Excel (3.9894228040143274E-16 vs ...328E-16)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: sonnet
defer: phase-1b
depends_on: []
---

## Goal
GAUSS(1E-15) returns Excel's 3.989422804014328E-16 instead of 3.9894228040143274E-16 (1 ulp).

## Context
Left over from T-2434 (z * phi(0) below 2E-15; statistics.md "GAUSS of a tiny argument"). gen-m9/a54c738c697617c3 Calc!A235 `=GAUSS(Data!C11)`; z*phi(0) and erf(z/sqrt2)/2 both give ...274E-16. Code: `fn_gauss` in functions/statistical/gauss.rs.

## Acceptance criteria
- [ ] gen-m9/a54c738c697617c3 Calc!A235 leaves eval/baseline/gen-m9.txt with no new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

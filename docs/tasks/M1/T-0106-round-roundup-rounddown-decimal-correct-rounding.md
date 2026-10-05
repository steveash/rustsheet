---
id: T-0106
title: "ROUND/ROUNDUP/ROUNDDOWN decimal-correct rounding per Excel"
milestone: M1
lane: engine
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-0009]
---

## Goal
Match Excel for binary-inexact halves (ROUND(1.005,2), ROUND(2.675,2)).

## Context
Hypothesis in docs/excel-semantics/numbers.md; probe cases pending oracle.

## Acceptance criteria
- [ ] gen-m2 ROUND-family last-digit cells leave eval/baseline/gen-m2.txt (T-0203 triage): 1b6ceed7489e0800 A178; 2cfe279fe77a000d A79 A120 A307 A382; 75840b9c71a50596 A389; 78bc88199fc96650 A387; 8eb18149d4245e1a A267; b4392a6514a36ade A148; d0c29dd5c530e28c A237; dd99be1f2454159f A14; ef5c55a81783b7d5 A325 (Calc sheet).
- [ ] probes/precision-and-rounding A6–A8 pass bit-exact; no regressions; semantics entry VERIFIED with implementation link.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

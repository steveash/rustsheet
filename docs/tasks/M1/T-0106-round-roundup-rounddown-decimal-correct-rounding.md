---
id: T-0106
title: "ROUND/ROUNDUP/ROUNDDOWN decimal-correct rounding per Excel"
milestone: M1
lane: engine
status: done
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
- [x] gen-m2 ROUND-family last-digit cells leave eval/baseline/gen-m2.txt (T-0203 triage): 1b6ceed7489e0800 A178; 2cfe279fe77a000d A79 A120 A307 A382; 75840b9c71a50596 A389; 78bc88199fc96650 A387; 8eb18149d4245e1a A267; b4392a6514a36ade A148; d0c29dd5c530e28c A237; dd99be1f2454159f A14; ef5c55a81783b7d5 A325 (Calc sheet).
- [x] probes/precision-and-rounding A6–A8 pass bit-exact; no regressions; semantics entry VERIFIED with implementation link.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
ROUND/ROUNDUP/ROUNDDOWN now round in decimal (`decimal_round.rs`): cut to 15 significant digits,
then round at `num_digits`, converting the decimal result to the nearest double. A new oracle
probe (probes/round-family, 50 cells) showed the 16th-digit tie rule differs by function (ROUND
toward zero, ROUNDUP/ROUNDDOWN away) and that a ROUNDUP overflow is `#NUM!`; numbers.md records
both. Eval delta: gen-m2 known failures 807 → 785 (22 fixed, every listed cell except
dd99be1f2454159f A14), probes A6 fixed and round-family 52/53; no regressions in any suite.
dd99be1f2454159f A14 is `ROUND(97.5,1)^-2.5`: ROUND is exact there; the last-digit miss is `^`
(T-0221). Follow-up: T-0141 (round-family A20, MOD of a huge quotient is `#NUM!`).

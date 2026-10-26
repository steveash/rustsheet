---
id: T-2778
title: "IMSINH's sinh kernel on the web: IMSINH(4) rounds up, IMSINH(-4) does not (gen-m9, 34 cells)"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-2535]
---

## Goal
IMSINH gives Excel for the web's digits for a real part where its sinh differs from libm's: `IMSINH("4")` is
27.2899171971278 while `IMSINH("-4")` is -27.2899171971277.

## Context
Split from T-2535, which found the task's premise wrong for IMSINH (engineering.md "sinh and cosh inside IMSIN and
IMCOS are (EXP(t) ∓ EXP(-t))/2, IMSINH's are not"):
- Web evidence: probes/t2535-imsinh-imcosh-kernel-sweep (IMSINH and IMCOSH of 62 inputs, excel-graph 2026-10-26) and
  probes/t2535-complex-text-tie (the complex text is not the cause: `COMPLEX((EXP(4)-EXP(-4))/2,0)` prints ...277,
  `IMREAL(IMSINH(4))` is 27.2899171971278, so IMSINH's double at 4 is 27.289917197127753, one ulp above
  `(EXP(4)-EXP(-4))/2` = 27.28991719712775, which is also libm's sinh and SINH(4)).
- libm `sinh` (the engine now) misses only x = 4 of the sweep's 43 real arguments (and the 34 gen-m9 cells below).
  `(EXP(x)-EXP(-x))/2` with Excel's EXP misses 4 and 0.01; cosh - EXP(-x) misses 12 and 0.01; a correctly rounded
  sinh misses -4, ±12 and 12-3i, and turns ironcalc-calc COMPLEXs.xlsx P12 and P21 (Mac desktop cache, `IMSINH(-12)`,
  `IMSINH("12-3i")`) into failures; the web agrees with desktop there. IMCOSH matches libm cosh on all 62.
- No symmetric formula of EXP(x) and EXP(-x) gives both 4 and -4: look for an asymmetric x87 sequence (an unstored
  EXP in a register for one operand), probe a dense sweep around 4 and both signs first (excel-probe skill;
  docs/research/excel-kernel-playbook.md).
- Put IMSIN and IMCOS at imaginary parts ±4, ±12 and 0.01 in the same probe: T-2535's EXP form for them rests on 4
  gen-m9 cells and never saw the points where it and libm differ.
Owned gen-m9 cells (T-2258 triage): the `IMSINH("4")` and `IMSEC(IMSINH("4"))` lines of T-2535's list (34 of its
38), plus probes/t2535-imsinh-imcosh-kernel-sweep A23 (`IMSINH("4")`) and probes/t2535-complex-text-tie A7, A14:A16,
A18 (IMSINH at 4), known failures since T-2535.
Code: `imsinh` in functions/engineering/complex/trig.rs.

## Acceptance criteria
- [ ] The gen-m9 `IMSINH("4")` / `IMSEC(IMSINH("4"))` known-failure lines leave eval/baseline/gen-m9.txt, with no new
      known-failure lines in any baseline (COMPLEXs.xlsx P12, P21 stay passing).
- [ ] The kernel and its evidence are recorded in engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IMSIN and IMCOS (T-2535), the overflow edge near 710 (T-2231).

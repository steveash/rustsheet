---
id: T-1940
title: "EXP F2XM1 near-tie residuals: test the non-Intel F2XM1 hypothesis (T-1931)"
milestone: M9
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1931]
---

## Goal
Explain, or reproduce, Excel's EXP at the F2XM1 near-ties that model V6 misses: decide whether the 87 cells of
probes-numeric `t1931-exp-ties` (and the 20 ERFC cells of `t1916-erfc-exp-ties`) follow a rule the engine can
model, or stay documented residuals.

## Context
- Model V6 (docs/excel-semantics/numbers.md, "EXP is an x87 2^(a*log2 e) evaluation, not correctly rounded";
  `crates/rustsheet-engine/src/x87_exp.rs` `excel_exp`) takes F2XM1 as RN64(2^|f| - 1). At points where 2^|f| - 1
  is within 0.09 ulp of a 64-bit midpoint Excel's F2XM1 rounds the other way: `t1931-exp-ties` holds all 87 such
  cells of `kmyst-erfc-r5expmap1/2`, each a 1 ulp miss (engine minus Excel +1 ulp at 47, -1 ulp at 40).
- The T-1916 re-fill changed 9 of its 20 ERFC near-tie cells to the engine's value, while the T-1931 re-fill
  (2026-10-19) returned all 87 EXP cells unchanged: the CPU-dependence hypothesis (an AMD Zen F2XM1) is untested.
- The T-1778 round-8 notes describe a hardware test (`hwtest.c`) for a non-Intel F2XM1.

## Acceptance criteria
- [ ] A model of the near-tie rounding that reproduces the `t1931-exp-ties` cells without losing any cell of
  `t1931-exp-map`/`-random`/`-gauss`/`-neg`, or a numbers.md record of why the cells stay residuals (with the
  re-fill stability evidence).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing EXP away from model V6 elsewhere.

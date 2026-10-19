---
id: T-2022
title: "EFFECT computes (1+r/n)^n with Excel's integer power kernel"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
EFFECT returns Excel's bits by raising `1 + r/n` to the (floored, integer) `npery` with the same
right-to-left square-and-multiply Excel uses for `^`, instead of libm `powf`.

## Context
Gap triage 2026-10-19. All 10 EFFECT known failures are last-digit or near-last-digit misses
(NumericUlp/NumericClose):
- eval/baseline/ironcalc-calc.txt, `ironcalc-calc/FINANCIAL/NOMINAL_EFFECT.xlsx` Sheet1 D2, F2, D3, F3, D4,
  F4, F8, D17, F17 (e.g. F2 `EFFECT(C2,B2)`: Excel 0.5000000000000013, ours 0.5000000000000004).
- eval/baseline/probes.txt, probes/fin-argument-modes Sheet1!H125 `EFFECT(0.05,4)`: Excel
  0.050945336914062445, ours 0.05094533691406222 (also listed in T-1374; this task takes it over).
`fn_effect` (crates/rustsheet-engine/src/functions/financial/rate_conversions.rs) computes
`(1.0 + nominal_rate / npery).powf(npery) - 1.0`. Triage check: `(1+r/n)` raised by right-to-left
square-and-multiply in plain doubles, then `- 1`, reproduces all 20 cached EFFECT cells of
NOMINAL_EFFECT.xlsx and H125 (libm `pow` 11/20, left-to-right square-and-multiply 15/20,
`expm1(n*log1p(r/n))` 6/20). The kernel is VERIFIED for `^` (docs/excel-semantics/numbers.md "Integer
exponents: right-to-left square-and-multiply") and lives in `arithmetic::power`
(crates/rustsheet-engine/src/arithmetic.rs); reuse it, do not copy the loop. Its products are rounded
twice (x87 model); if that changes any of the 21 cells, report which and compare with the plain-double
loop before choosing. Rule recorded in docs/excel-semantics/financial.md "EFFECT and NOMINAL".
NOMINAL is not this task (its non-integer power is a fit: T-2024).

## Acceptance criteria
- [ ] The 10 EFFECT lines above disappear from eval/baseline/ironcalc-calc.txt and eval/baseline/probes.txt
      (`make eval-baseline`), with no new F lines in any baseline.
- [ ] A unit test pins `EFFECT(0.05,4)` = 0.050945336914062445 and one NOMINAL_EFFECT.xlsx cell.
- [ ] financial.md "EFFECT and NOMINAL" says EFFECT is implemented (Ours line), citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NOMINAL (T-2024); argument validation and lifting (unchanged).

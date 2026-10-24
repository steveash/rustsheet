---
id: T-2566
title: "XNPV and XIRR discount with Excel's ^ (probes/xirr-exact-root-sign)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XNPV, and the XNPV that XIRR evaluates at its trial rates, discount each value with Excel's own `^`, so
probes/xirr-exact-root-sign matches. Its XNPV cells sit at an exact root, where only the last ulps and the sign
are left. XIRR then picks Excel's bracket side at an exact chain root (xirr-span-zero-first A11, A14 and A20).

## Context
T-2328 (financial.md, "XNPV discounts with Excel's `^`") found that Excel's XNPV is
sum value_i / (1 + r)^((d_i - d_0)/365) with Excel's `^`: integer exponents by square-and-multiply, the others
by exp(y·ln x). The lab model fits all 541 numeric probe cells. `compute_xnpv` and `xirr_xnpv` in
crates/rustsheet-engine/src/functions/financial/financial_util.rs use `powf`. Reuse
`arithmetic::power` (or whichever variant the worksheet `^` uses) instead of writing a new power. Keep
`xirr_xnpv`'s rule that a power which overflows or underflows to 0 has no value.

## Acceptance criteria
- [ ] probes/xirr-exact-root-sign's XNPV cells pass, and its XIRR cells keep passing; the
      xirr-span-zero-first A11, A14 and A20 known failures are fixed (`make eval-baseline`).
- [ ] No regression across the XNPV and XIRR eval cases. Unit test on one exact-root cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's stop rule (already fitted, T-1583/T-2328).

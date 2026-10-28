---
id: T-2566
title: "XNPV and XIRR discount with Excel's ^ (probes/xirr-exact-root-sign)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: []
campaign: visible
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
- [ ] probes/xirr-exact-root-sign's XNPV cells pass, and its XIRR cells G4, G5 and G7 pass (the
      same flows as xirr-span-zero-first A11, A14 and A20, also fixed; `make eval-baseline`). Its column J
      only shows that XNPV is `#NUM!` for a date at serial 0, unlike XIRR (financial.md).
- [ ] No regression across the XNPV and XIRR eval cases. Unit test on one exact-root cell. Optionally, register an
      `xnpv` fitlab target over the probe so the lab's 541/541 is rechecked by a command.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's stop rule (already fitted, T-1583/T-2328).

## Visible-parity stage
Target (decision 0114): every visibly-off cell this task owns is within half a unit of the 15th significant digit
of Excel's value, or has Excel's type, error and text. Bit-level criteria still unmet then move to a follow-up
filed with `defer: phase-1b`.

Phase 1 triage (T-0090, eval of 2026-10-27 at `main` 7f6b373a, task ownership as of `main` e5bc60c4): this task owns 183 of the 185 visibly-off XNPV cells (probes/xirr-exact-root-sign) and 6 of the 14 XIRR cells. Fixing them lifts XNPV to the Phase 1 rate (≥ 99.9% of attributed cells within 15 digits); no other open task owns enough of those cells.

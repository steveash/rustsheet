---
id: T-1843
title: "XNPV's rate, XIRR's guess and FVSCHEDULE's principal lift over arrays beside their whole data (SPEC-0011)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1822, T-1373]
---

## Goal
XNPV's rate, XIRR's guess and FVSCHEDULE's principal lift over arrays and refuse a multi-cell range, with their cash flows, dates and schedule read once, whole.

## Context
Split from T-1822. probes/fin-argument-modes: `ROWS(XNPV({0.09;0.09},K1:K5,M1:M5))`, `ROWS(XIRR(K1:K5,M1:M5,{0.1;0.1}))`
and `ROWS(FVSCHEDULE({1000;1000},{0.1,0.2}))` are 2 (H341, H338, H137), while a range there is `#VALUE!` (H342, H339,
H138): each lifted argument is the function's only scalar, so it is fully probed. The flows, dates and schedule are
`WholeByCaller` (SPEC-0011; their read rules are T-1373's): read once, captured by the kernel, each element charged
their cells (`LiftCost::Cells`, `lift_call_costed`, as SERIESSUM's coefficients in T-1802 and T-1821's NPV/IRR).
XIRR's solver walks and bisects with fixed caps (financial.md "The IRR and XIRR solver caps") over n flows, so its
entry is `Weighted` by those caps.

## Acceptance criteria
- [ ] H137, H338 and H341 of probes/fin-argument-modes pass (baseline lines removed); H138, H339, H342 still pass.
- [ ] A test shows the flows are evaluated once for a lifted call, and a lifted XIRR over large flows is refused by
  the work bound before the first element.
- [ ] `make check` green; reviewers approve.

## Out of scope
- How the flows and schedule are read (T-1373); solver parity (T-0916).

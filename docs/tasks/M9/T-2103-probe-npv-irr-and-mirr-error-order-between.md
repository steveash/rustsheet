---
id: T-2103
title: "Probe NPV, IRR and MIRR error order between the rate and the cash flows, and the shape of a lift over erroring flows"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for which error wins when both the rate (guess) and the cash flows of NPV, IRR and MIRR are errors,
and for the shape of a lifted call whose cash flows are an error.

## Context
T-1821 lifted NPV's rate, IRR's guess and MIRR's rates beside cash flows read once. It kept the old scalar order:
NPV checks its rate before the flows, and IRR and MIRR check their flows before the guess or rates. A flows error is
every lifted element's error. No probe backs this (docs/excel-semantics/financial.md "NPV, IRR and MIRR error order",
UNVERIFIED). Hand-written tests: `test_fn_financial_lift.rs`
`npv_irr_and_mirr_lift_their_rates_beside_whole_cash_flows` H10-H14. Probe for each function: a text rate or guess
with an error in the flows, scalar and over `{"x";0.1}`. Also probe `SUM(--ISERROR(IRR(F4,{0.1;0.1})))` with F4 =
`=1/0`, which tells an array of errors from one scalar error.

## Acceptance criteria
- [ ] An oracle-filled probe case covers the cells above; the financial.md entry is VERIFIED or corrected, with a
  follow-up task for any engine mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up does).

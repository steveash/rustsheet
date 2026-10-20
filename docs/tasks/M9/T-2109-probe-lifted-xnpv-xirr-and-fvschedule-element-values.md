---
id: T-2109
title: "Probe lifted XNPV, XIRR and FVSCHEDULE element values and error order beside erroring flows"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the element values of lifted XNPV, XIRR and FVSCHEDULE and for which error wins when both the
lifted scalar (rate, guess, principal) and the flows, dates or schedule are errors.

## Context
T-1843 lifted XNPV's rate, XIRR's guess and FVSCHEDULE's principal beside flows read once. The oracle cells only show
the shapes (probes/fin-argument-modes H137-H138, H338-H339, H341-H342). The element values and the error order rest
on hand-written tests: `test_fn_financial_lift.rs` `xnpv_xirr_and_fvschedule_lift_beside_their_whole_flows` H3, H6,
H9-H15 (docs/excel-semantics/financial.md "XNPV, XIRR and FVSCHEDULE beside their whole data", order unprobed).
Probe, scalar and over `{"x";0.1}` (FVSCHEDULE: `{TRUE;1000}`): `INDEX(XNPV({0.09;0.2},K1:K5,M1:M5),2)`,
`INDEX(XIRR(K1:K5,M1:M5,{0.1;0.2}),2)`, `INDEX(FVSCHEDULE({1000;2000},{0.1,0.2}),2)`, and each with an error cell
(`=1/0`) in the flows, dates or schedule beside a text rate or guess or a boolean principal; plus
`SUM(--ISERROR(XIRR(F4,M1:M5,{0.1;0.1})))`, which tells an array of errors from one scalar error. Pairs with T-2103.

## Acceptance criteria
- [ ] An oracle-filled probe case covers the cells above; the financial.md entry's order is VERIFIED or corrected,
  with a follow-up task for any engine mismatch.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a follow-up does).

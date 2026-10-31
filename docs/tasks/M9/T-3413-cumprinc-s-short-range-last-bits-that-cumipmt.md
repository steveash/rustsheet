---
id: T-3413
title: "CUMPRINC's short-range last bits that CUMIPMT's PMT·m - CUMPRINC amplifies (T-3118's regressed and still-off cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3118]
campaign: visible
---

## Goal
Our CUMPRINC has Excel's bits on the short ranges whose CUMIPMT is still off, so those interests are right too.

## Context
Since T-3118 CUMIPMT is `PMT·m - CUMPRINC` (financial.md "CUMIPMT and CUMPRINC over many periods"),
and our PMT has Excel's bits (T-3014). Every CUMIPMT cell still off is our CUMPRINC 1 to 4 ulps from
the x that solves `PMT·m - x = Excel's CUMIPMT`, amplified by the cancellation (an ulp of a principal
of 1,000 is hundreds of ulps of an interest of 1 to 100). Cells: probes/fin-cumipmt-gate-edges A1-A4,
A6, A7 (type 0, rates 6E-5 to 1.5E-4, periods 3-10; x is -3 to +1 ulps from ours), E3-E10 (type 1,
8E-6 to 1E-4; -4 to +2 ulps), H3; fin-cumipmt-overflow-and-forms A8, A12 (x one ulp below our
-1570.8656559792591), A14 (+2 ulps); fin-cumipmt-tiny-rate-structure F2 (x is Excel's own probed
CUMPRINC G2); CUMPRINC_CUMIPMT.xlsx G2, G3; gen-m9 6e18550e9fc9c1b9 A329, 79d87fc527927626 A352.
T-3118's regressed cells are listed in decision 0197. The probes that pair the two functions
(tiny-rate-structure) carry CUMPRINC's own expected values for the fit.

Added by gap triage 2026-10-31: the other visibly-off CUMIPMT cells of tiny-rate-structure have the same cause. Each
column-A CUMIPMT sits beside our column-B CUMPRINC, which is 1-2 ulps from Excel's (A10 `CUMIPMT(2E-5,100,100000,12,12,1)`
Excel -1.7801601439258548, ours -1.7801601439259684; B10 CUMPRINC Excel -999.2101533464493, ours -999.2101533464491).
- probes/fin-cumipmt-tiny-rate-structure.case.json (37): Sheet1!A2, A4, A5, A8, A9, A10, A11, A12, D5, D6, D8, D11, F1, F3,
  F5, F6, F8, F9, F10, F12, H2, H3, H8, K2, K4, K8, K9, K10, K11, O10, O11, Q10, Q11, S6, S7, S8, S10

## Acceptance criteria
- [ ] The listed CUMIPMT cells are bit-exact through a CUMPRINC change, or financial.md records why
      the remaining ones are not.
- [ ] No regressions (`make eval-baseline`); `make check` green; reviewers approve.

## Out of scope
- CUMPRINC near period 1E15 (T-3141) and the bond/depreciation bits of T-1374.

## Visible-parity stage
Decisions 0083 and 0114 (`visible`; key added by gap triage 2026-10-31 when the 37 tiny-rate-structure cells joined):
done when the owned CUMIPMT cells are within half a unit of the 15th significant digit of Excel's value. Bit parity on
the rest is Phase 1b (decision 0188 R5). Time-boxed by decision 0056.

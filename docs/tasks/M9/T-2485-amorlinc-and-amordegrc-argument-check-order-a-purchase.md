---
id: T-2485
title: "AMORLINC and AMORDEGRC: argument check order, a purchase date below 1 and fractional periods (gen-m9, 41 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: visible
---

## Goal
AMORLINC and AMORDEGRC give Excel's result for error, text and boolean arguments next to out-of-domain ones, accept a purchase date below serial 1 as Excel does, and round a fractional period as Excel does.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. Three behaviours:
- Order: a boolean rate or salvage is `#VALUE!` before a negative cost or a salvage above the cost is `#NUM!`
  (`AMORLINC(-439103,37926,39037,412,7,FALSE,..)`, `AMORDEGRC(27029,37050,39005,789,1E15-0.1,TRUE)` are `#VALUE!`, ours
  `#NUM!`), while a date past 9999-12-31 or a negative rate given before an error argument is `#NUM!`
  (`AMORDEGRC(57226,2958466,..,<#VALUE!>,0)`, `AMORLINC(15926,37905,39408,352,13,-1,<#VALUE!>)` are `#NUM!`, ours `#VALUE!`).
- A purchase date below 1 is valid: `AMORLINC(46557,0,39881,440,0,0.1)` = 46117, `AMORLINC(16348,1e-15,38048,0.01,3,3.1E26,1)` = 0,
  `AMORLINC(71868,0.0358..,39873,484,0,0.2,4)` = 71384 (ours `#NUM!`).
- `AMORDEGRC(98882,37287,38647,1E-13,0.5625,0.1,1)` is 1 (ours 0): a fractional period.
The order is inferred from the cells (docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)"); probe the bare calls before changing the reader (excel-probe
skill). Cells that nest LINEST, LOGEST or XIRR are owned by T-1741, T-1742 and T-2428, not here.
Code: crates/rustsheet-engine/src/functions/financial/amortization.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 41 known-failure lines in eval/baseline/gen-m9.txt, 41 visibly off.
Examples:
- gen-m9/23d82b62e9a85bdd Calc!A309 `=AMORLINC(16348,1e-15,38048,0.01,3,EXP(61),1)`: Excel 0.0, ours #NUM!
- gen-m9/31474c20fc1dafd6 Calc!A317 `=AMORLINC(26359,37616,38400,522,9,Data!D8)`: Excel #VALUE!, ours 0.0
- gen-m9/abb7d565af89b849 Calc!A52 `=AMORLINC((-439103),37060,2.5,98,0,Data!$A12,TRUE)`: Excel #NUM!, ours #VALUE!
- gen-m9/b73c6cc96e093f03 Calc!A164 `=AMORLINC(77223,37475,0.5,Data!D8,10,((Data!D1-Data!$E12)/1),0)`: Excel #VALUE!, ours #NUM!
- gen-m9/e8d55ec393460bb7 Calc!A318 `=AMORDEGRC(Data!A2,37043,39932,610,8,0.2,Data!B8)`: Excel #NUM!, ours #VALUE!
- gen-m9/e8d55ec393460bb7 Calc!A360 `=AMORDEGRC((-1),37167,38781,Data!E$7,2,0)`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/01523d247594952e.case.json (3): Calc!A230, A255, A370
- gen-m9/0a957119475760c5.case.json (2): Calc!A75, A111
- gen-m9/1b60bf99d71424e7.case.json (2): Calc!A298, A368
- gen-m9/1cd3096e3e87c965.case.json (2): Calc!A113, A308
- gen-m9/1dce4774f5c0e736.case.json (3): Calc!A36, A101, A248
- gen-m9/23d82b62e9a85bdd.case.json (2): Calc!A309, A370
- gen-m9/30e20ba3302fea8b.case.json (1): Calc!A371
- gen-m9/31474c20fc1dafd6.case.json (3): Calc!A121, A317, A357
- gen-m9/5422bdbf173fc233.case.json (2): Calc!A83, A242
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A381
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A36
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A204
- gen-m9/81bcdc0bec0c8e6b.case.json (1): Calc!A321
- gen-m9/89f0aee953a55a45.case.json (2): Calc!A72, A219
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A226
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A17
- gen-m9/abb7d565af89b849.case.json (1): Calc!A52
- gen-m9/b6a02893a8d6b151.case.json (1): Calc!A139
- gen-m9/b73c6cc96e093f03.case.json (2): Calc!A164, A360
- gen-m9/bc04c1f478c9896c.case.json (2): Calc!A267, A278
- gen-m9/d912671baa1364aa.case.json (1): Calc!A76
- gen-m9/e168be7f6d3f84c2.case.json (1): Calc!A148
- gen-m9/e8d55ec393460bb7.case.json (5): Calc!A164, A290, A302, A318, A360

## Acceptance criteria
- [x] Visible parity (gen-m9, T-2257): these 41 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline. (40 of 41; d912671baa1364aa A76 moved to T-1895, see Outcome)
- [x] Unit tests for a purchase date of 0 and 1E-15, a boolean rate beside a negative cost, and a period of 0.5625 (moved to T-1895, see Outcome).
- [x] `make check` green; reviewers approve.

## Out of scope
- Periods past 2^20 and the work cap (T-1895).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed first (probes/amor-argument-check-order, 866 cells, excel-graph): both functions read cost to rate in order, a date
off 0-2958465 being `#NUM!` as it is read, then check the values, then read the basis, and AMORDEGRC checks its asset life
last; a purchase on the first period's end is valid, and a purchase date below 1 is serial 0, the day before serial 1.
`amortization_of` follows it (with the shared `day_count_basis_of` and `financial_calendar_date_of`) and `amor_date` maps
serial 0. Every order cell matches (financial.md "AMORLINC and AMORDEGRC check order").
A period between 0 and 1 is not a rounding: AMORDEGRC returns a value left over from another evaluation
(probes/amordegrc-fractional-period), which depends on the calculation order; the one owned cell that shows it,
gen-m9/d912671baa1364aa Calc!A76, and the probe lines move to T-1895, which already owned AMORDEGRC below period 1, so the
0.5625 unit test is not written here.
Eval delta: gen-m9 -23 known failures (40 of the 41 owned cells pass; 17 had already left the baseline), probes 0 fixed and
21 new-coverage lines in the two new cases (amor-argument-check-order A846-A851, amordegrc-fractional-period A1-A11 and
A23-A26), all stale-period cells owned by T-1895; 0 regressions.

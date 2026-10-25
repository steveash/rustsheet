---
id: T-2486
title: "COUP* functions: a settlement or maturity past 9999-12-31 is #NUM! as it is read, and the calendar-edge day counts (gen-m9, 20 cells)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
The COUP* functions are `#NUM!` for a settlement or maturity past 9999-12-31 as soon as that argument is read (before a later error, text or basis), and COUPDAYS and COUPDAYBS count Excel's days at the calendar edge and with basis 4.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. financial.md "Analysis ToolPak securities functions check frequency and
dates first" (T-1406, VERIFIED) has the per-argument order but only a settlement below 1 as a read-time `#NUM!`. These cells
add the upper bound: `COUPDAYS(123456789.123,<FALSE>,4)`, `COUPNUM(37484,2958466,2,<#VALUE!>)`, `COUPDAYS(38153,<123456789.123>,
4,TRUE)` and `COUPNUM(38810,1E15+1,4.9,<#VALUE!>)` are `#NUM!` (ours `#VALUE!`), and `COUPDAYS(39123,2958466,1,0)`,
`COUPDAYS("2024-02-29",1E15,4,2)` and `COUPDAYS(39654,7.55E40,4.9,3)` are `#NUM!` where ours returns 360, 90 and 91.25.
Day counts: `COUPDAYBS(37913,46720,4,4)` is 50 (ours 51) and `COUPDAYS(5E-05,2958465,2.5,1)` is 184 (ours 181).
Some cells have an error argument from DGET, DSTDEV, DAVERAGE, GROWTH or ACCRINT whose Excel value may be another; if a probe
shows the inner call differs, move the cell to T-2496. docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: `coupon_args` and the coupon schedule in crates/rustsheet-engine/src/functions/financial/financial_bonds.rs and
financial_bonds/coupon_schedule.rs.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 20 known-failure lines in eval/baseline/gen-m9.txt, 20 visibly off.
Examples:
- gen-m9/5c6a23c4f6597f1c Calc!A86 `=COUPPCD(ACOSH(DGET(Db!A2:D9,2,Db!F1)),46568,4)`: Excel #VALUE!, ours #NUM!
- gen-m9/7d2979fba2c95d64 Calc!A304 `=COUPDAYBS(37913,46720,4,4)`: Excel 50.0, ours 51.0
- gen-m9/a91bad4e84f49739 Calc!A153 `=COUPDAYS(39123,2958466,1,0)`: Excel #NUM!, ours 360.0
- gen-m9/a91bad4e84f49739 Calc!A398 `=COUPDAYS(38153,Data!$A7,4,TRUE)`: Excel #NUM!, ours #VALUE!
- gen-m9/b72d952c5f52cb4d Calc!A88 `=COUPDAYS((Data!B7*(-0.5)),2958465,Data!B1,1)`: Excel 184.0, ours 181.0
- gen-m9/ba4ddb817a8e700e Calc!A65 `=COUPNCD(38368,44562,DSTDEVP(Db!B1:D9,3,Data!B8:B8),4)`: Excel #VALUE!, ours #NUM!
Cells:
- gen-m9/1cd3096e3e87c965.case.json (2): Calc!A187, A362
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A19
- gen-m9/5c6a23c4f6597f1c.case.json (1): Calc!A86
- gen-m9/5ded7594e597e31e.case.json (1): Calc!A391
- gen-m9/6376e27be7a05871.case.json (1): Calc!A36
- gen-m9/79d87fc527927626.case.json (1): Calc!A59
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A304
- gen-m9/89f0aee953a55a45.case.json (2): Calc!A265, A363
- gen-m9/9a1fa5366bc50918.case.json (2): Calc!A13, A372
- gen-m9/9d9ceba257f8001b.case.json (1): Calc!A249
- gen-m9/a91bad4e84f49739.case.json (3): Calc!A153, A161, A398
- gen-m9/b72d952c5f52cb4d.case.json (1): Calc!A88
- gen-m9/ba4ddb817a8e700e.case.json (1): Calc!A65
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A204
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A274

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 20 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Unit tests: a maturity of 2958466 before an error basis, a settlement of 123456789 before a boolean maturity, and the two day counts.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 1904 coupon rules (T-2125, T-2337).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

## Outcome
Probed the rules (probes/coup-calendar-edges, 129 cells, and coup-calendar-edges-1904, 8 cells; excel-graph,
2026-10-25) and implemented two of them:
- A COUP* settlement or maturity that does not floor to a day of the workbook's calendar is `#NUM!` as it is read
  (`coupon_date_of` in functions/financial/dates.rs; in a 1904 workbook the bound is the 1904 calendar's last day).
- Each coupon date is the maturity moved back whole periods, its day clamped, so a 29th/30th maturity no longer drifts
  to the 28th after a February (`coupon_period` in coupon_schedule.rs; COUPDAYS' basis-1 29 February walk keeps the
  chained walk, `chained_coupon_period`). PRICE, YIELD, DURATION, MDURATION and ODDF* read the same schedule; their
  probe cells (J1-J14) confirm the change for them too.
Eval: 11 gen-m9 known failures fixed (5674 -> 5663; 8 of the 20 owned cells had already been fixed before this task);
probes: two new cases; the new-coverage known-failure lines are the 15 settlement-0 basis-1 COUPDAYS cells (owned by
T-2632) and the yield-solver last digits of J2, J8 and J12 (NumericClose, about 1e-14 relative: the "YIELD stops short
of PRICE's root" gap), accepted knowingly against the "no new known-failure lines" criterion. Follow-up probe of the
ODDL* and ACCRINT walks: T-2639. The one owned cell left,
gen-m9/b72d952c5f52cb4d Calc!A88 (`COUPDAYS(5E-05,2958465,2,1)` = 184), follows an irregular pattern recorded in
financial.md "COUPDAYS basis 1 for a period before 1900-01-01" and moves to T-2632.

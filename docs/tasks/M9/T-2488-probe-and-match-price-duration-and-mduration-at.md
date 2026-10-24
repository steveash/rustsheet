---
id: T-2488
title: "Probe and match PRICE, DURATION and MDURATION at a huge yield or coupon: #NUM!, not a number (gen-m9, 10 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
PRICE, DURATION and MDURATION return Excel's `#NUM!` at a huge yield or coupon rate, where ours returns a number, with the boundary established by a probe.

## Context
Gap triage of the T-2257 gen-m9 financial sweep. Excel gives `#NUM!` and ours a number for `PRICE(39050,44044,0.5,123456789.123,
110,4,2)` (ours -3.89), `PRICE(37245,44043,2147483647,1E+300,102,1,0)` (ours -8.4E10), `PRICE(37232,45792,0,1E+300,102,4)`
(ours 0), `DURATION(36810,44793,1E+10,1E+15,1,2)` (ours 0.856), `DURATION(366,46685,42,136,2)` (ours 0.324),
`MDURATION(38368,44642,0.01,1E+300,4,3)` (ours 7.3E-301), `MDURATION(38166,46616,4.35E14,1E15,4)` (ours 5.4E-16). The cells
do not show where the boundary is (an overflow of `(1+yld/frequency)^n`, a yield cap, or a coupon cap), so the task first
probes PRICE, DURATION and MDURATION over yields and coupons from 1 to 1E300 (excel-probe skill; a few dozen cells), records
the rule in financial.md, then implements it. The YIELD counterpart is T-2391. docs/excel-semantics/financial.md "Financial functions in the gen-m9 financial sweep (T-2257)".
Code: crates/rustsheet-engine/src/functions/financial/financial_bonds.rs (`fn_price`, `fn_duration`, `fn_mduration`).

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 10 known-failure lines in eval/baseline/gen-m9.txt, 10 visibly off.
Examples:
- gen-m9/1dce4774f5c0e736 Calc!A8 `=PRICE(39050,44044,0.5,SUM(123456789.123),110,4,2)`: Excel #NUM!, ours -3.888802290922487
- gen-m9/67a4c03afd63f988 Calc!A217 `=DURATION(366,46685,42,COMBIN(17,2),2)`: Excel #NUM!, ours 0.3240196078431373
- gen-m9/c0681a8b150ce301 Calc!A248 `=MDURATION(38368,44642,1%,1E+300,4,3)`: Excel #NUM!, ours 7.2602739726027396e-301
Cells:
- gen-m9/1b60bf99d71424e7.case.json (1): Calc!A345
- gen-m9/1dce4774f5c0e736.case.json (1): Calc!A8
- gen-m9/3aa168259335be54.case.json (1): Calc!A364
- gen-m9/6376e27be7a05871.case.json (1): Calc!A294
- gen-m9/67a4c03afd63f988.case.json (1): Calc!A217
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A343
- gen-m9/9a1fa5366bc50918.case.json (1): Calc!A148
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A176
- gen-m9/c0681a8b150ce301.case.json (1): Calc!A248
- gen-m9/d0854366830dcdee.case.json (1): Calc!A153

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): these 10 known-failure lines disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The probe case is in eval/suites/probes with excel-graph values and its rule is in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- YIELD (T-2391).

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): every cell this task owns has the wrong type, error, text or value (none is a
last-digit difference), so the task is in the campaign. Its target is that every cell it owns has Excel's type, error and text,
or is within half a unit of the 15th significant digit of Excel's value; the visible-parity criterion under Acceptance criteria
states it.

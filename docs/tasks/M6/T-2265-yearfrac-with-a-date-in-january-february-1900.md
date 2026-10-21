---
id: T-2265
title: "YEARFRAC with a date in January-February 1900 (serials 1-60): count on the displayed calendar (ours a day off)"
milestone: M6
lane: functions-date
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
YEARFRAC gives Excel's value when a date is in January or February 1900 (serials 1-60, fractions included): on basis 0 the date is the displayed one (serial 1 is 1900-01-01, serial 60 is 29 February without the end-of-February adjustment), and basis 1 matches the two cells too.

## Context
Gap triage of gen-m6 (T-2180, excel-graph oracle, 2026-10-21). Cells and observations: docs/excel-semantics/dates.md "Serials below 61 in
DATEDIF, EDATE, DAYS360, WEEKNUM and YEARFRAC" (added by this triage). `year_fraction_parts` (crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs)
reads dates through `financial_1900_serial`/`date_of_serial`, the weekday-faithful stand-in, so a date in serials 1-59 is a day early
(basis 0 is 1/360 high) and serial 60 is 28 February (adjusted to day 30, so 1/360 low: `YEARFRAC(55862,60)` = 55000/360 in Excel, 54999/360 ours).
Basis 1 (2 cells): `YEARFRAC(48344,1,1)` = 132.35660175388034 and `YEARFRAC(25538,1,1)` = 69.91786287010599; fit the year-length average over
1900 with the fictitious leap day. 25 cells are basis 0. The discount-security functions share `year_fraction_parts` (financial.md
"Discount securities: day count and year length"): no financial cell may move, or list the ones that do.
T-2237 handles serials in [0, 1) in the same function; coordinate (whichever lands second rebases). Probe if the cells do not pin the
rule (excel-probe: basis 0-4 for s in {1, 31, 59, 60, 60.5, 61} against e in {366, 45000}, about 60 cells).
Why opus: fitting basis 1's year length and the end-of-February rule at serial 60.

Failing cells (`make eval-show CASE=gen-m6/<case>.case.json`), 27 in eval/baseline/gen-m6.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m6/0bcac2d96e96b3df Calc!A83 | `=YEARFRAC(3.14159,33767)` | 92.44166666666666 | 92.44444444444444 |
| gen-m6/0bcac2d96e96b3df Calc!A121 | `=YEARFRAC(DAY(18610),15655)` | 42.825 | 42.827777777777776 |
| gen-m6/0bcac2d96e96b3df Calc!A387 | `=YEARFRAC(Data!B$2,28916)` | 79.16944444444445 | 79.17222222222222 |
| gen-m6/121a7c17092038d9 Calc!A175 | `=YEARFRAC(22676,7,0)` | 62.06388888888889 | 62.06666666666667 |
| gen-m6/1a404a0379a1372c Calc!A305 | `=YEARFRAC(MONTH(15659),15833)` | 43.32222222222222 | 43.325 |
| gen-m6/216e3c05f407ca79 Calc!A290 | `=YEARFRAC(Data!$B4,52900,(SUBTOTAL(Data!A5,Data!E6:E11,Data!E7:E10)/SUM(Data!D4:D9,Data!$D7)))` | 144.71944444444443 | 144.72222222222223 |
| gen-m6/2ba5bd794e7e8987 Calc!A63 | `=YEARFRAC(48344,Data!B4,1)` | 132.35660175388034 | 132.3572727458472 |
| gen-m6/3516b9fbbd60269b Calc!A35 | `=YEARFRAC(IF(Data!B4,Data!D$6,LEN(Data!E3)),42911)` | 117.48333333333333 | 117.48611111111111 |
| gen-m6/3e51213f35889d06 Calc!A320 | `=YEARFRAC(55862,60)` | 152.77777777777777 | 152.775 |
| gen-m6/44194b2fefa30de2 Calc!A270 | `=YEARFRAC(SUMPRODUCT(Data!B8:E11),28535,0)` | 78.10833333333333 | 78.11111111111111 |
| gen-m6/4ca39e0876796fa5 Calc!A308 | `=YEARFRAC(60.5,34993)` | 95.64444444444445 | 95.64166666666667 |
| gen-m6/535caabe1e635e2d Calc!A289 | `=YEARFRAC(1.005,17870)` | 48.922222222222224 | 48.925 |
| gen-m6/752959b8d3db0285 Calc!A35 | `=YEARFRAC(1,30145)` | 82.53333333333333 | 82.53611111111111 |
| gen-m6/752959b8d3db0285 Calc!A225 | `=YEARFRAC(59,27182)` | 74.25555555555556 | 74.26388888888889 |
| gen-m6/7f3606fef4cb9df7 Calc!A53 | `=YEARFRAC(MUNIT(1),53144)` | 145.5 | 145.50277777777777 |
| gen-m6/7f3606fef4cb9df7 Calc!A205 | `=YEARFRAC(44957,ABS((Data!B12-Data!C3)))` | 123.07777777777778 | 123.08055555555555 |
| gen-m6/7f3606fef4cb9df7 Calc!A395 | `=YEARFRAC(3.14159,45291)` | 123.99444444444444 | 123.99722222222222 |
| gen-m6/9e6a1b733ee1e1e0 Calc!A268 | `=YEARFRAC((SUM(Data!E3,Data!C9:C10,(-1))^Data!C8),5789,0)` | 15.847222222222221 | 15.85 |
| gen-m6/acc70339aac84a23 Calc!A223 | `=YEARFRAC(IF(ISERROR(#NUM!),DAY(51777)),24284)` | 66.48055555555555 | 66.48333333333333 |
| gen-m6/b62dd1ebd266b71c Calc!A385 | `=YEARFRAC(POWER(2^31,(Data!C3*1e-15)),58221)` | 159.40277777777777 | 159.40555555555557 |
| gen-m6/be0e3a1b0fc3f7a0 Calc!A231 | `=YEARFRAC(16112,1.005)` | 44.108333333333334 | 44.111111111111114 |
| gen-m6/c4edd76f46980c5f Calc!A334 | `=YEARFRAC(3.14159,44530)` | 121.90833333333333 | 121.91111111111111 |
| gen-m6/d0c30225243fde60 Calc!A38 | `=YEARFRAC(25538,CELL("col",Data!A5),1)` | 69.91786287010599 | 69.91851766157643 |
| gen-m6/d0c30225243fde60 Calc!A190 | `=YEARFRAC(15065,((Data!D4-(-0.5))-Data!B2))` | 41.24722222222222 | 41.25 |
| gen-m6/da0289bc74cdf6af Calc!A270 | `=YEARFRAC(59052,(Data!A3/0.5))` | 161.6638888888889 | 161.66666666666666 |
| gen-m6/da0289bc74cdf6af Calc!A384 | `=YEARFRAC(AVERAGE(Data!C11:C12),23563)` | 64.5111111111111 | 64.51388888888889 |
| gen-m6/ef10bf3afb9658f6 Calc!A368 | `=YEARFRAC(53701,7)` | 147.00555555555556 | 147.00833333333333 |

## Acceptance criteria
- [ ] These 27 known-failure lines disappear from eval/baseline/gen-m6.txt (`make eval-baseline`), with no new known failure in any suite; no financial cell changes.
- [ ] dates.md states YEARFRAC's rule for serials 1-60 per basis with its cells, and drops the UNVERIFIED note for basis 1 if probed.
- [ ] Unit tests in test/test_yearfrac_basis.rs cover serial 1, 59, 60 and 60.5 on basis 0 and 1.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Serials in [0, 1) (T-2237).
- DATEDIF, EDATE, DAYS360 (T-2263).

## Visible-parity stage
Decision 0083: every cell above is visibly off (a wrong error, a wrong type or a value off in a leading digit). The acceptance criteria above are the stage.

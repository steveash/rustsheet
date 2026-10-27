---
id: T-2173
title: "ODD and EVEN of values above 2^53: fit Excel's last digit (ODD(9999999999999999))"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
campaign: visible
depends_on: [T-2172]
---

## Goal
ODD and EVEN return Excel's doubles for values above 2^53 (`ODD(9999999999999999)` = 9999999999999992).

## Context
Gap triage of gen-m5 (T-2114, excel-graph oracle, 2026-10-20). Needs an oracle sweep: T-2172. `fn_odd`/`fn_even` (mathematical.rs) compute `ceil((|x|-1)/2)*2+1` and `ceil(|x|/2)*2` in
doubles; above 2^53 `|x|-1` already rounds. Candidate models to test against the sweep: the 15-digit cut first (ROUND's
stage 1, decimal_round.rs), ROUNDUP's "adds the unit in binary" (numbers.md), `2*ceil((x+1)/2)-1`. Observation:
docs/excel-semantics/numbers.md "ODD and EVEN above 2^53: last digits not explained". The EVEN cell's input comes from SERIESSUM
and may itself be an ulp off in Excel (T-2163), so fit on the sweep, not on that cell alone.

Failing cells (`make eval-show CASE=gen-m5/<case>.case.json`), 3 in eval/baseline/gen-m5.txt:

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/38b1b8e51dbad326 Calc!A10 | `=EVEN(((0.1/Data!E$5)*SERIESSUM((-2.5),Data!B10,3,0.1)))` | 6.223015277861144e+37 | 6.223015277861143e+37 |
| gen-m5/6b831f5e2a20bc34 Calc!A283 | `=ODD(9999999999999999)` | 9999999999999992.0 | 9999999999999988.0 |
| gen-m5/bbbd316c346ce645 Calc!A224 | `=RADIANS((ODD(9999999999999999)*((-1)-Data!B11)))` | -209439510239319.38 | -209439510239319.3 |

gen-m5 round-2 triage (T-2252, excel-graph oracle, 2026-10-22): one more ODD cell. Ours computes EVEN of the literal (which reads
9999999999999990) as 9999999999999990, so the ODD is the same call as Calc!A283 above and the innermost divergence, unless
Excel's EVEN differs there too (the T-2172 sweep's EVEN points tell).

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/d88d12a62b726a0a Calc!A232 | `=ODD(EVEN(9999999999999999))` | 9999999999999992.0 | 9999999999999988.0 |

gen-m5 round-3 triage (T-2648, excel-graph oracle, 2026-10-25): one more ODD cell, and this one is visibly off. With Excel's
ODD of 9999999999999992, AVERAGE gives 4999999999999996, whose 15-digit cut is 5E+15, so CEILING.PRECISE is 5E+15; ours
averages our 9999999999999988 to 4999999999999994, which cuts to 4999999999999990. Our CEILING.PRECISE of 4999999999999996 is
already 5E+15, so the ODD is the innermost divergence.

| case / cell | formula | Excel | ours |
|---|---|---|---|
| gen-m5/36133d9266f8ed26 Calc!A144 | `=CEILING.PRECISE(AVERAGE(Data!E8,ODD(9999999999999999)))` (E8 = 1) | 5000000000000000.0 | 4999999999999990.0 |

## Acceptance criteria
- [ ] These 5 known-failure lines (3 from T-2114, 1 from T-2252, 1 from T-2648) disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Every ODD/EVEN cell of the T-2172 sweep passes, or the remaining ones are listed residuals in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SERIESSUM (T-2163).

## Visible-parity stage
Decision 0083/0114: this task is done when every visibly-off cell it owns has Excel's type, error and text, or is within half a unit of the 15th significant digit of Excel's value. Bit-level criteria still unmet then move to a follow-up task filed without the key and with `defer: phase-1b`.
Gap triage 2026-10-25 (eval on origin/main becefc6f3): the task was deferred by decision 0114's adoption, but an eval shows a cell it owns visibly off, so, as its Deferred note asked, the `defer` key gave way to `campaign: visible`. Visibly off: gen-m5/36133d9266f8ed26 Calc!A144 `=CEILING.PRECISE(AVERAGE(Data!E8,ODD(9999999999999999)))`: Excel 5000000000000000, ours 4999999999999990 (the M5 gen suite's visibly-off cell; ODD is the innermost divergence, Context).

---
id: T-2173
title: "ODD and EVEN of values above 2^53: fit Excel's last digit (ODD(9999999999999999))"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
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

## Acceptance criteria
- [ ] These 3 known-failure lines disappear from eval/baseline/gen-m5.txt (`make eval-baseline`), with no new known failure in any suite.
- [ ] Every ODD/EVEN cell of the T-2172 sweep passes, or the remaining ones are listed residuals in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SERIESSUM (T-2163).

---
id: T-1540
title: "TBILLEQ up to 182 days with a zero denominator is 0"
milestone: M2
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-1475]
---

## Goal
`TBILLEQ(1000,1100,3.6)` (100 days, `360 - rate*DSM` = 0) is 0, as Excel gives, not `#DIV/0!`.

## Context
Known failure probes/tbilleq-zero-price Calc!A3 (excel-graph). The ≤182-day branch of `fn_tbilleq`
(crates/rustsheet-engine/src/functions/financial/mod.rs) is `365*rate/(360 - rate*DSM)`; Excel gives 0
at a zero denominator. Probe a negative denominator first (excel-probe): `TBILLEQ(1000,1100,4)`,
`TBILLEQ(1000,1182,2)`, and the same with TBILLPRICE, to find the rule (a price ≤ 0 → 0? only = 0?). Also probe a negative
price from 183 days whose square-root argument stays positive, `TBILLEQ(1000,1183,2)` and
`TBILLEQ(45332,45650,60)` (we give finite negative yields, UNVERIFIED), and match what Excel gives.
Rule: docs/excel-semantics/financial.md "TBILLEQ and TBILLYIELD".

## Acceptance criteria
- [x] probes/tbilleq-zero-price A3 leaves eval/baseline/probes.txt; the new probe values are recorded.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 183-day-plus branch (T-1475).

## Outcome
Probed (probes/tbilleq-nonpositive-price, excel-graph): TBILLEQ is `#NUM!` for every negative price (both
branches), 0 at a zero denominator up to 182 days; TBILLPRICE is 0 at price 0 and `#NUM!` when negative.
`fn_tbilleq` now matches (zero denominator is 0, negative price is `#NUM!` instead of a finite negative
yield). Eval delta: probes/tbilleq-zero-price A3 fixed, new case 9/9 bit-exact, regressions 0.

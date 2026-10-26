---
id: T-2815
title: "Lifted ODDFYIELD and AMORDEGRC charged the cap's bound: #ERROR! where Excel spills (gen-m9, 2 cells)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
A lifted ODDFYIELD or AMORDEGRC over an array of many elements that the kernel refuses before any work returns its
array, as Excel does (`#SPILL!` in these cells' layout), instead of `#ERROR!` from the per-call lift budget, while each
lifted call keeps a provable work bound (directive 5).

## Context
Split from T-2795, which charged the incomplete-gamma entries by value and fixed their four cells. The two left:
- gen-m9/ba4ddb817a8e700e Calc!A323 `=ODDFYIELD(45535,45661,(SQRTPI(3.14159)-AVERAGE(..)),38502,(FILTER(Data!B1:C12,..)-(Data!E6/3.14159)),82,110,4)`:
  22 elements, all errors in `rate`, and the issue date is about -1.2E14. `ValueWork::CouponWalks` (lift/kinds/mod.rs)
  clamps a negative earliest date to the longest schedule, so each element is charged `YIELD_SOLVER_WEIGHT` (2^24, 8
  elements a call). But `odd_args` (financial_bonds.rs) returns `#NUM!` for a date off the calendar as it reads it, and
  an error in any argument before any walk. So the ODD* entries could charge 1 for a date outside [0, 2958466) or for an
  error in any argument. Check the order in `odd_args` first, then
  probes/fin-odd-date-before-error-arg and odd-bond-argument-check-order.
- gen-m9/e8d55ec393460bb7 Calc!A10 `=AMORDEGRC(56947,_xlfn.PIVOTBY(..),39763,716,2,PMT(..),1.9)`: about 1,250
  elements, about 1,029 of them texts, against `AMORDEGRC_WEIGHT` (2^18, 512 elements). A text `date_purchased` that
  cannot read as a number is `#VALUE!` before the walk. But `ArgValue::Text` carries only a length, so a by-value charge
  needs a "may read as a number" bit (e.g. no digit in it). Check what the kernel coerces first, and what the
  walk's length follows from (cost, salvage, rate: financial.md "AMORLINC and AMORDEGRC work cap").

## Acceptance criteria
- [ ] Visible parity (gen-m9): the 2 known-failure lines above disappear from eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] Each changed charge is a bound of its element's work, and the financial.md DEVIATION entries say how an element
      is charged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The incomplete-gamma entries (T-2795).

## Visible-parity stage
Decision 0083: both cells have the wrong error, so the task is in the campaign.

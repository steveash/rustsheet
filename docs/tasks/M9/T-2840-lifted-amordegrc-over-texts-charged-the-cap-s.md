---
id: T-2840
title: "Lifted AMORDEGRC over texts charged the cap's bound: #ERROR! where Excel spills (gen-m9, 1 cell)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-2815]
campaign: semantics
---

## Goal
A lifted AMORDEGRC over many elements its kernel refuses before the walk returns its array (Excel `#SPILL!` in this
cell's layout) instead of `#ERROR!` from the per-call lift budget, while each element keeps a provable work bound
(directive 5).

## Context
Split from T-2815, which charged the ODD* elements refused before their walk. Left:
gen-m9/e8d55ec393460bb7 Calc!A10 `=AMORDEGRC(56947,_xlfn.PIVOTBY(..),39763,716,2,PMT(..),1.9)`, about 1,250 elements,
about 1,029 of them texts, against the flat `AMORDEGRC_WEIGHT` (2^18, 512 elements; lift/kinds/financial.rs).
`Model::amortization_of` (functions/financial/amortization.rs) reads cost to rate left to right, returning on an error, a
logical, a text that does not read as a number, or a date off the calendar, all before the walk; then the value checks
and the asset life. A by-value charge (`KernelWork::ByValue`) could charge such an element 1, but `ArgValue::Text`
carries only a length, so it needs a sound "may read as a number" bit: show which texts `Model::cast_number` can never
read (e.g. no numeric character at all; check `parse_excel_number`, `parse_number_text` with currencies, dates with month
names and times such as "6 PM", and coercion.md "Number words") with a test over the engine's parser, not a guess.
The walk's own length follows from cost, salvage, rate and period (financial.md "AMORLINC and AMORDEGRC work cap").

## Acceptance criteria
- [ ] Visible parity (gen-m9): the Calc!A10 line of gen-m9/e8d55ec393460bb7 leaves eval/baseline/gen-m9.txt
      (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] The charge is a bound of each element's work, and financial.md's DEVIATION entry says how an element is charged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ODD* charges (T-2815).

## Visible-parity stage
Decision 0083: the cell has the wrong error, so the task is in the campaign.

---
id: T-3483
title: "Probe where gen-m9/d912671baa1364aa A76's AMORDEGRC below period 1 gets Excel's 1: does an AMORDEGRC that fails its asset-life check leave a value?"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
Find which earlier formula leaves Excel's 1 for the AMORDEGRC below period 1 in gen-m9/d912671baa1364aa Calc!A76,
record the rule in financial.md "AMORDEGRC below period 1", and make the engine return 1 there.

## Context
Gap triage 2026-10-31 (`make eval` on main b9558688e). The cell is visibly off and attributed to no function (an
AMORDEGRC over SUMPRODUCT), and no open task owns it:
- gen-m9/d912671baa1364aa.case.json (1): Calc!A76 `AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`,
  where the period is 0.5625. Excel gives 1, ours 0.

financial.md "AMORDEGRC below period 1" (T-1895, T-2825, T-2844): at a period strictly between 0 and 1 AMORDEGRC returns
a value left behind by the last AMORDEGRC or AMORLINC Excel evaluated before it, or 0. T-2485, T-2825 and T-2844 all
left this cell "unexplained". The workbook's other AMOR cells and Excel's values: A27
`AMORDEGRC(57838,37283,39648,592,9,10,4)` `#NUM!`; A47 AMORLINC `#NUM!`; A96 `AMORLINC(92598,36663,39379,343,6,1E-300,3)`
9.259800000000001E-296 (its first-period charge rounds to 0, so it cannot leave 1); A125, A145, A194, A223, A243,
A272 `AMORDEGRC(14888,37512,39953,478,5,1,4)`, A321 `#NUM!`; A174, A390 `#N/A`; A292, A341, A370 `#VALUE!`.
Hypothesis to test first: an AMORDEGRC that passes every argument check and fails only the asset-life check, which
comes last (financial.md "AMORLINC and AMORDEGRC check order"), still writes the left-over state. That fits A27 (rate
10) and A272 (rate 1). A rate of 1 or 10 gives a degressive charge that the 1 could be the rounded form of. Other
candidates are an error raised after the first charge, or Excel's calculation order across the sheet (T-3393 studies
the order around dynamic-array formulas).

## Acceptance criteria
- [ ] A probes case `eval/suites/probes/cases/t3483-amordegrc-left-over-after-error.case.json`, filled with
      excel-graph (excel-probe skill). Each workbook holds one AMORDEGRC at period 0.5625 and, before it, one of
      A27's, A272's, A47's and A96's formulas alone, and then all of them in sheet order. Variants change A27's and
      A272's rate so the life check passes, to tell the asset-life hypothesis apart from the calculation order.
- [ ] financial.md "AMORDEGRC below period 1" states the rule with the new cells as evidence, and marks
      d912671baa1364aa A76 explained.
- [ ] The engine follows the rule: A76's known-failure line disappears from eval/baseline/gen-m9.txt, with no new line
      in any baseline (`make eval-baseline`). If the rule needs calculation-order modelling beyond what
      `amor_left_over` does today, the Outcome files that as its own engine task instead, and this criterion moves there.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AMORDEGRC's left-over state around dynamic-array formulas (T-3393).

## Visible-parity stage
Decisions 0083 and 0114 (`semantics`): done when A76 is Excel's 1, or its follow-up is filed with the rule recorded.

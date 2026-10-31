---
id: T-3409
title: "Explain AMORDEGRC's left-over 1 at gen-m9 d912671baa1364aa A76 by bisecting its workbook on excel-graph"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
gen-m9/d912671baa1364aa Calc!A76 is explained: the probe evidence names which cell (or which rule) leaves Excel's
AMORDEGRC left-over value at 1 for this period-0.5625 call, financial.md "AMORDEGRC below period 1" states the rule, and
the engine gives 1 there. If the rule turns out to be an order effect the engine cannot reproduce within decision
0131's model, the task ends with that evidence and a decision-backed exception instead of a fix.

## Context
- Gap triage 2026-10-31c (opus refill). The cell has had no open owner since T-1895 and T-2485 closed (both list it;
  T-2485 moved it to T-1895, which handed the rule to T-2825). financial.md "AMORDEGRC below period 1" (T-2825, T-2844)
  ends with: "gen-m9 d912671baa1364aa A76 (period 0.5625) is 1, which the table does not explain" and marks it
  UNEXPLAINED. `make exit-owners MILESTONE=M9` does not list it because the cell's formula also calls SUMPRODUCT and
  TIMEVALUE, so it is not one of AMORDEGRC's attributed cells; it is still a visibly-off known failure in
  eval/baseline/gen-m9.txt.
- The cell (excel-graph, 2026-10-28): `=AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)`
  (salvage 1E-13, period 0.5625): Excel 1, ours 0.
- The rule so far (financial.md, VERIFIED on freshly loaded workbooks): a fractional-period AMORDEGRC returns the value
  the last AMORDEGRC or AMORLINC that Excel's fresh-load order (recalc.md) ran before it left; errors and period-0 calls
  leave the value as it was, AMORLINC leaves its first-period charge rounded to a whole number, a lifted call leaves the
  state empty, and a dynamic-array formula's place in the order is only partly explained (T-3393, probes -ag and -ah).
- What the workbook holds (excel-graph values from gen-m9/d912671baa1364aa.oracle.json): every other AMORDEGRC or
  AMORLINC of Calc is an error (A27, A47, A125, A145, A194, A223, A243, A272, A321 `#NUM!`; A174, A390 `#N/A`; A292,
  A341, A370 `#VALUE!`) except A96 `AMORLINC(92598,36663,39379,343,6,1E-300,3)` = 9.2598E-296, which by the table
  leaves 0 (its first-period charge rounds to 0; probes/amordegrc-fraction-state-r). None of them leaves 1. The sheet also
  holds many dynamic-array formulas (A2 FILTER, A32 TOROW, A63 WRAPROWS, A73 SORTBY, A82 HSTACK, A95 UNIQUE, A167 and A239
  SEQUENCE, A343 MAKEARRAY with a LAMBDA, and more), which is where T-3393's open question sits.
- Code: `Model::amortization_call` and `LeftOver` (crates/rustsheet-engine/src/functions/financial/amortization.rs),
  `OrderState::amor_left_over` (evaluation.rs), the order pass in excel_order.rs (SPEC-0017, decision 0131).

Plan (evidence first; no fit to the single cell):
1. Bisect on excel-graph: copy the case (Calc and Data sheets) into probe cases that drop halves of the Calc cells other
   than A76 (keep A76 and the Data sheet), fill them, and narrow down to the cell or cells whose presence makes A76 1.
   Five or six rounds of halving cover the 399 other cells; keep each round's probe as a committed case.
2. State the rule in financial.md with the probe cells, and implement it if it fits the existing order model. If it
   is the T-3393 dynamic-array question, coordinate with that task (whichever lands first records the rule; the other
   re-triages in its Outcome).

Cells:
- gen-m9/d912671baa1364aa.case.json (1): Calc!A76

## Acceptance criteria
- [ ] The bisection probes are committed under eval/suites/probes/cases/ (excel-graph filled, one milestone tag M9),
      and the Outcome names the cell or rule that leaves 1 for A76.
- [ ] financial.md "AMORDEGRC below period 1" no longer calls A76 UNEXPLAINED: it states the rule with the probe cells.
- [ ] gen-m9/d912671baa1364aa Calc!A76 leaves eval/baseline/gen-m9.txt (`make eval-baseline`) with no new known-failure
      lines in any baseline, and the new probe cases pass; or, if the rule cannot be reproduced, the Outcome says why and a
      decision record names the cell as an exception (decision 0087 / 0134 style), with the cell still a known failure.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The dynamic-array placement of probes/amordegrc-fraction-state-ag and -ah: T-3393 (fold the result in if the
  bisection answers it, and say so in both tasks).
- AMORLINC or AMORDEGRC values at period 1 or above (no failing cell).

## Visible-parity stage
Decision 0083 and 0114 (`semantics`): done when A76 has Excel's value (1), or when the evidence shows it needs a
recorded exception. A wrong number from a calculation-order state rule is a semantic rule, not a numeric kernel.

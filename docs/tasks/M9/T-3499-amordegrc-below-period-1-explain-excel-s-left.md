---
id: T-3499
title: "AMORDEGRC below period 1: explain Excel's left-over 1 at gen-m9 d912671baa1364aa A76, which the order table gives as 0"
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
Excel's value of `AMORDEGRC(98882,37287,38647,SUMPRODUCT((1e-15*Data!A10)),TIMEVALUE("1:30 PM"),0.1,1)` at
gen-m9/d912671baa1364aa Calc!A76 (1) is explained by a probed rule, recorded in financial.md, and the engine returns it.

## Context
- Gap triage 2026-10-31 (opus refill, `make eval` on origin/main): the only visibly-off AMORDEGRC cell in a
  user-representative suite, and no open task owns it (`scripts/exit_owners.py --all-off` lists it as `unattributed`).
  T-2825 (decision 0131) and T-2844 modelled the left-over value below period 1 and left this cell UNEXPLAINED.
- Cell list:
  - gen-m9/d912671baa1364aa.case.json (1): Calc!A76 (Excel 1, ours 0)
- The period is `TIMEVALUE("1:30 PM")` = 0.5625, so AMORDEGRC returns the left-over value of the call before it in
  Excel's fresh-load order (financial.md "AMORDEGRC below period 1"; SPEC-0017: last sheet first, bottom row first,
  right to left, a precedent first). financial.md says the table does not explain this cell: the last AMORDEGRC or
  AMORLINC run before it, A96 `AMORLINC(92598,36663,39379,343,6,1E-300,3)`, leaves 0 (probes/amordegrc-fraction-state-r),
  and the case's other AMORDEGRC and AMORLINC calls (A27, A47, A125, A145, A174, A194, A223, A243, A272, A292, A321,
  A341, A370, A390) are errors or values that do not leave 1.
- Hypotheses to probe (excel-probe, excel-graph, one fresh workbook per question, as T-2825 did):
  1. A96's own salvage is 343 and its rate 1E-300: its rounded first-period charge may be 1 rather than 0 under
     basis 3, so the state it leaves differs from fraction-state-r's call; probe A96 alone, then A96 with A76 below it.
  2. The salvage argument `SUMPRODUCT((1e-15*Data!A10))` (1E-13) or the TIMEVALUE period changes which state is read
     (a lifted or array-evaluated argument runs on a state of its own since T-2844); probe A76 with a literal salvage
     and a literal 0.5625.
  3. A non-AMOR function of the case touches the state (WEEKNUM shares `OrderState` with it); probe the case's other
     rows above A76 in a copy with A76's precedents kept.
- Code: `Model::amortization_call` (functions/financial/amortization.rs), `OrderState::amor_left_over`
  (evaluation.rs), `excel_order.rs`.
- Related: T-3393 (the dynamic-array formula's place in the order, probes -ag and -ah). If the probes show the same
  mechanism, fold the fix into one PR and name both tasks.

## Visible-parity stage
`semantics`: the value is a state rule, not arithmetic. Target: Calc!A76 equals Excel's 1, with no other cell of
gen-m9 or the amordegrc probes moved off.

## Acceptance criteria
- [ ] A probe case (excel-graph filled) isolates what leaves 1 before A76; financial.md "AMORDEGRC below period 1"
      replaces "gen-m9 A76 is UNEXPLAINED" with the rule and cites the probe cells.
- [ ] gen-m9/d912671baa1364aa Calc!A76 passes: its F line leaves eval/baseline/gen-m9.txt with no new known-failure
      line in any suite (`make eval-baseline`).
- [ ] If the rule cannot be found within the time box (decision 0056), the Outcome lists the hypotheses tested with
      their probe cells, financial.md records them, and the cell stays a known failure with a follow-up task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The dynamic-array formula cases -ag and -ah (T-3393), unless the same rule explains them.
- AMORDEGRC past 2^22 periods (T-2934).

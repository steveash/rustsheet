---
id: T-2665
title: "Probe IPMT with a fractional nper past (1+rate)^nper's range: exact balance or FV recurrence"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IPMT with a fractional nper past `(1+rate)^nper`'s range (P above about 1E+270) has Excel's form, decided by probe
evidence, and the engine has one rule for integer and fractional terms there.

## Context
T-2490 moved IPMT past the range to the exact balance `-rate*B` (÷(1+rate) for type 1; docs/excel-semantics/financial.md
"IPMT cancels on fv"), which fits 91 of 118 integer-nper, non-cancelling type-1 cells of probes/fin-ipmt-ppmt-huge-rate
to the bit (the FV recurrence on the rounded PMT 2). The one fractional-nper cell probed there, gen-m9
0a957119475760c5 Calc!A153 `=IPMT(1.005,7,~1.85E+7,92160,674,1)`, is Excel -46194.91271820449: the old FV
recurrence's value, and `-rate*(B/(1+rate))`, not the exact balance's -46194.91271820448. So `period_payments`
keeps the FV recurrence for a fractional nper past the range (`whole_term` in
crates/rustsheet-engine/src/functions/financial/annuity.rs). Probe: IPMT type 0 and 1 at rates 0.01 to 1E+10 with
fractional and integer nper of 1E3 to 1E8 past the range, pv and fv mixed; decide whether the split is on the
fractional term, on the type-1 operation order, or on something else, and drop the `whole_term` special case if it
is not Excel's.

## Acceptance criteria
- [ ] A probe case with oracle values (excel-graph) for the grid above, and its finding in financial.md.
- [ ] gen-m9 0a957119475760c5 Calc!A153 stays passing; no new known-failure lines.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IPMT's cancelling pv-0 cells (financial.md "IPMT cancels on fv", Open).

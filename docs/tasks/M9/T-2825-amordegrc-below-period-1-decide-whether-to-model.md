---
id: T-2825
title: "AMORDEGRC below period 1: decide whether to model Excel's left-over value (calculation-order state)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
campaign: semantics
depends_on: [T-1895]
---

## Goal
A recorded decision on AMORDEGRC with 0 < period < 1, whose Excel value is whatever the previous AMORDEGRC evaluation
left behind (0 alone in a fresh workbook), and the cells it owns either matched or kept as known failures under that
decision.

## Context
T-1895 probed each such formula alone in a fresh workbook (probes/amordegrc-fraction-alone-a to -g, excel-graph
2026-10-26): every one is 0, our value; next to an `AMORDEGRC(..,2,..)` of 485 it is 485 when Excel evaluates the 485 cell
first, and Excel for the web evaluated those sheets from the last cell to the first (docs/excel-semantics/financial.md
"AMORDEGRC below period 1"). The value depends on Excel's calculation order and on state carried between cells, which the
engine has neither (its evaluation order is the dependency graph's, and functions are pure). Options for the decision:
keep them known failures (the formula alone matches); or carry a per-recalculation "last AMORDEGRC value" and evaluate in
Excel's order (only meaningful if that order is knowable; probe a sheet with the cells in another order, other sheets,
and a recalculation after an edit first). Also unexplained: gen-m9/d912671baa1364aa Calc!A76 (period 0.5625) is 1 though
every other AMORDEGRC in that workbook is an error, so the state may be shared with other functions (probe: an AMORLINC,
another ATP function or a plain number cell below a fractional-period AMORDEGRC).
Cells (known failures): probes/amordegrc-fractional-period A1-A11, A23-A26; probes/amor-argument-check-order A846-A851;
probes/fin-amor-period-bound B17; probes/amordegrc-fraction-alone-f A1; gen-m9/d912671baa1364aa Calc!A76.

## Acceptance criteria
- [ ] A decision record (`make new-decision`) choosing between modelling the state and keeping the cells as known failures,
  citing the probes; financial.md "AMORDEGRC below period 1" updated to match.
- [ ] If modelled: the cells listed under Context pass with no new failures; otherwise their baseline lines stay, named in
  the entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The AMORDEGRC walk's arithmetic past 2^20 periods (T-2826).

---
id: T-2619
title: "IRR is #NUM! on four gen-m9 cash-flow sets where ours returns a rate (T-2475 probe)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
IRR is `#NUM!` on the owned cash-flow sets where Excel finds no rate, instead of the huge or near -1 rate ours returns.

## Context
Found by T-2475's probe, probes/t2475-stat-subexpressions (excel-graph, 2026-10-24): each owned gen-m9 cell's innermost diverging sub-expression, evaluated on a copy of the case's Data sheet (D01..; the probe's column B names the cell). Errors are shown as `#<ERROR.TYPE>` (2 `#DIV/0!`, 3 `#VALUE!`, 4 `#REF!`, 6 `#NUM!`, 7 `#N/A`). Record each rule in docs/excel-semantics/ with the probe cells as evidence.

Owned gen-m9 cells (4; known failures in eval/baseline/gen-m9.txt, and the probe's cells in eval/baseline/probes.txt):
- gen-m9/0620505280d2ea6e.case.json Calc!A77: probe P!A2 `IRR(D01!C4:E7,-D01!C8)`: Excel "#6", ours 31622775.851604085
- gen-m9/92d4fa29e5510f1e.case.json Calc!A236: probe P!A330 `IRR(D26!C8:E12,1E-10)`: Excel "#6", ours 399999999999998.7
- gen-m9/a63d1c7d6f96c56d.case.json Calc!A111: probe P!A424 `IRR(D33!C4:E8,((-0.5)*D33!D4))`: Excel "#6", ours -0.9710878190154174
- gen-m9/eedd629f3d69fdda.case.json Calc!A297: probe P!A563 `IRR(D47!A6:C8)`: Excel "#6", ours -0.8583707295763425

Related: T-1686 (IRR caps and fallback path probe).

## Acceptance criteria
- [ ] The owned gen-m9 cells leave eval/baseline/gen-m9.txt, with the probe's matching cells (`make eval-baseline`), and no new known-failure lines; each rule is in docs/excel-semantics/ with its evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Cells owned by the other T-2475 follow-ups (T-2618 to T-2623).

## Outcome (2026-10-26)
Fixed by T-2727 (IRR's far-root bound and fallback rule, financial.md): all four owned gen-m9 cells left the baseline.

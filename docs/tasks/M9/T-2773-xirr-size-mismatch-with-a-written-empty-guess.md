---
id: T-2773
title: "XIRR size mismatch with a written empty guess: probe XIRR(v,d,) vs XIRR(v,d) on mismatched ranges"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
XIRR with a written empty guess and mismatched value/date sizes gives Excel's result, whichever error that is.

## Context
Found in T-2688's review. `fn_xirr` (functions/financial/mod.rs) returns the size error only beside a number
guess: `number_guess = matches!(arguments.get(2), None | Some(CalcResult::Number(_)))`. A text, boolean or error
guess keeps `#VALUE!` (gen-m9/b6a02893a8d6b151 A226, b72d952c5f52cb4d A50). T-2688 made a written empty guess
(`CalcResult::EmptyArg`) read as the absent guess for its value (financial.md "XIRR (and IRR): an empty guess
argument is the default 0.1 for XIRR, guess 0 for IRR"). But the size gate still treats it as a non-number
guess. So `XIRR(A1:A3,B1:B2,)` may take a different error path from `XIRR(A1:A3,B1:B2)`. No probe covers this.

## Acceptance criteria
- [ ] A probe case (excel-graph, at least 8 cells) compares `XIRR(v,d,)`, `XIRR(v,d)`, `XIRR(v,d,E1)` (E1 empty)
      and `XIRR(v,d,0.1)` on values longer than dates, dates longer than values, and array constants of
      mismatched length. It is gated or baselined, and the finding goes in financial.md.
- [ ] The engine matches on every cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's bisection and its empty-guess value rule (T-2688).

## Outcome
Probe `probes/xirr-size-mismatch-empty-guess` (36 cells, excel-graph): every cell is `#NUM!`, whatever the guess
(written empty, absent, empty cell, 0.1) and whether or not the flows hold text or booleans. The engine matched
on the 20 numeric-flow cells but gave `#VALUE!` for a written empty or empty-cell guess over text/boolean flows;
fixed in `fn_xirr` (`number_guess`). Finding in financial.md. Eval delta: +36 passing probe cells and 4 gen-m9 known failures fixed (A231, A197, A355, A145).

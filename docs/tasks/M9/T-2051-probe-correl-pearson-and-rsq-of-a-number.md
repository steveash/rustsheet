---
id: T-2051
title: "Probe CORREL, PEARSON and RSQ of a number cell against a blank cell (gen-m9 fcf11f2ec6f7a59d A9 is #VALUE!)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for what CORREL, PEARSON, RSQ (and SLOPE, INTERCEPT, STEYX, COVAR for comparison)
give when a single number cell is paired with a blank cell, so gen-m9/fcf11f2ec6f7a59d Calc!A9 can be
explained and fixed (or filed) instead of guessed.

## Context
- Known failure (gap triage 2026-10-19): gen-m9/fcf11f2ec6f7a59d Calc!A9
  `(MONTH(CORREL(Data!D10,Data!A7))-DAY((Data!A4+Data!C10)))` with D10 = 2.5, A7 blank, A4 FALSE,
  C10 blank: Excel (excel-graph) `#VALUE!`, ours `#DIV/0!`. `DAY(FALSE+blank)` is `DAY(0)` = 0, so
  Excel's `#VALUE!` must come from `CORREL(2.5-cell, blank-cell)` (or from MONTH of its result).
  T-1723 records that a single point of two numbers (`CORREL(A1,B1)`, `CORREL(5,5)`) is `#DIV/0!`
  in Excel; no probe has a number against a blank, which leaves zero pairs.
- The gen-m9 suite is ungated, so this cell has no baseline line; it is listed in `make eval`'s
  report only.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph) with each of CORREL, PEARSON, RSQ, SLOPE,
      INTERCEPT, STEYX, COVAR, COVARIANCE.S over: a number cell and a blank cell (both orders), a
      blank and a blank, a number and an empty-text cell, a number and a FALSE cell, two 2-cell
      ranges where every pair has a blank, and `MONTH(CORREL(D10,A7))` itself; no pending cells.
- [ ] The rule is recorded in docs/excel-semantics/statistics.md with the cells; every cell the
      engine misses is filed as a task (functions-stat) or added to T-1723 if it is the same rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

---
id: T-2616
title: "gen-m9 4250f880ab640c49 A258: GAMMA.INV(YIELDDISC(FORECAST(blank known_y)..)) is #NUM! in Excel, #VALUE! ours"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
gen-m9/4250f880ab640c49 Calc!A258 matches Excel (`#NUM!`).

## Context
`=GAMMA.INV(YIELDDISC(FORECAST((-1),Data!A10,Data!D11),44817,98,110,(-0.5)),(SUM(Data!E8:E9,Data!A2:A12,Data!B10:B11)-42),(ROUND(59,Data!D8)*DATE(1969,Data!D6,Data!D2)))`:
Excel `#NUM!`, ours `#VALUE!` since T-2426 (A10 empty, D11 = " x "). Alpha is -38.55841. Two hypotheses: GAMMA.INV or
YIELDDISC reports a domain `#NUM!` before an error argument; or Excel's FORECAST is not `#VALUE!` for a blank known_y
against a text one-cell known_x, a combination the T-2255 evidence never shows. Probe with the `excel-probe` skill first.

## Acceptance criteria
- [ ] The cell's known-failure line disappears from eval/baseline/gen-m9.txt (`make eval-baseline`), nothing new.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The blank-reference rule itself (T-2426).

---
id: T-3407
title: "IRR returns #DIV/0! on long conventional series near -0.2 to -0.5 (9 hold-out cells; the engine is #NUM!)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
IRR returns `#DIV/0!` where Excel does on conventional cash flows, instead of `#NUM!`.

## Context
- probes-numeric/t3347-irr-conventional-holdout A123, A126, A130, A131, A132, A134, A135, A138 and A143 (9 cells of
  the neg-long stratum): long series (36-60
  flows, uneven or level, scales 1 to 1e10) after one outlay, exact root r = -0.23 to -0.47. Excel (excel-graph,
  2026-10-31) is `#DIV/0!`; the engine is `#NUM!`, with or without T-3347's stage. The other 15 cells of the stratum
  are `#NUM!` in both.
- A Python replica of the secant (T-3347 Outcome) shows that on 7 of the 9 the first secant step lands where NPV
  overflows (no finite value), and on A131 and A138 the secant goes flat at step 3 and both Newton runs overflow; the
  `#NUM!` cells of the stratum never overflow. A first guess: an overflowing NPV inside Excel's iteration surfaces as
  `#DIV/0!` instead of failing the run. Probe it with short series built to overflow at the first step, and with a
  guess that avoids it.
- docs/excel-semantics/financial.md "IRR on conventional flows near 0 and -10%" (the hold-out paragraph).
- Code: `irr_npv`, `irr_secant`, `compute_irr` (functions/financial/financial_util.rs).

## Acceptance criteria
- [ ] A probe case (excel-graph) that isolates when IRR is `#DIV/0!`, its cells listed here before the fix.
- [ ] The engine returns `#DIV/0!` on those cells and on the 9 hold-out cells, with no gated cell regressing.
- [ ] The rule in docs/excel-semantics/financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which series return a number (T-3406).

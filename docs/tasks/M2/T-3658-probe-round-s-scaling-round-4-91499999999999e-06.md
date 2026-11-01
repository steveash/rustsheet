---
id: T-3658
title: "Probe ROUND's scaling: ROUND(4.91499999999999E-06,8) is 1 ulp above the nearest double"
milestone: M2
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3555]
---

## Goal
Find out when Excel's ROUND returns a double other than the one nearest its decimal result. Record the rule in
docs/excel-semantics/numbers.md: either rewrite "ROUND one ulp above the nearest double (observation)", added by T-3555,
or fold it into "ROUND family rounds in two decimal stages". File the engine task if the engine differs.

## Context
- T-3555's Gate A table `eval/accuracy/tables/ROUND/s1` Calc!A61 (excel-graph, 2026-11-01; PR #2691) has
  `ROUND(4.91499999999999E-06,8)` = 4.9100000000000004E-06 in Excel. That is 1 ulp above 4.91E-06, the double nearest
  the decimal result, and the engine returns 4.91E-06. It is the only one of the 360 ROUND-family points where the
  engine is not bit-exact with Excel. Gate A's envelope holds it, so it is not a failing point.
- numbers.md "ROUND family rounds in two decimal stages" (VERIFIED) says ROUND returns the nearest double.
- A candidate mechanism, from the Excel-fidelity review of #2691: in doubles `491*1E-8` is 4.9100000000000004E-06 and
  `491/1E8` is 4.91E-06. So Excel may scale the kept digits by multiplying by 10^-n on some path, much as ROUNDUP
  adds its unit in binary (numbers.md "ROUNDUP adds the unit in binary").
- Probe a grid of kept-digit integers m × num_digits n (say m in 1..999 and n in 1..12, with small and large
  magnitudes). Compare Excel's ROUND against the nearest double, `m*10^-n` and `m/10^n`, and record which model
  matches every cell. Use the `excel-probe` skill. ROUNDDOWN at the same points is the control.

## Acceptance criteria
- [ ] A probe case with oracle values separates the models; numbers.md states the rule (VERIFIED) with the cells.
- [ ] If the engine differs from the rule, an engine task is filed with the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the Gate A tables (a batch never changes) or the math definition (decision 0223).

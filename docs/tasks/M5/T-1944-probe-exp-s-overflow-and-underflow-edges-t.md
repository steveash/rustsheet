---
id: T-1944
title: "Probe EXP's overflow and underflow edges (T-1931)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-1931, T-2203]
---

## Goal
Establish with oracle evidence what Excel's EXP does at its range edges, and make the engine match.

## Context
- Evidence first (planner, 2026-10-20): the probe this task asks for is T-2203 (research), which fills `probes-numeric/t2203-exp-range-edges`. Use that case; do not probe again. Its failing cells are this task's cells too.
- `EXP_SIGN.xlsx` B44 is no longer this task's cell. Excel keeps 2^-1074 there (probes/subnormal-function-results C4), so the cached 0 is a file artifact; T-2204 (research) decides how it counts.
- Since T-1931 the worksheet EXP is model V6 (`x87_exp::excel_exp`; docs/excel-semantics/numbers.md "EXP is an
  x87 2^(a*log2 e) evaluation, not correctly rounded"). V6 has not been probed at the edges of its range.
- Overflow: a in (709.7827, 709.79), where libm's exp and V6 could place the last finite double and `#NUM!`
  differently. Also probe `EXP(710)`.
- Underflow: a near -745.13, where V6's single rounding to the subnormal grid decides between 2^-1074 and 0. Use
  `(x)=0` companions, because Graph reports a kept subnormal as 0.0 (decision 0042).
- Use the `excel-probe` skill. Put a dense grid of consecutive doubles, as stored inputs, around each edge.

- Existing cell (gap triage 2026-10-19): ironcalc-calc/MATH_AND_TRIGONOMETRY/EXP_SIGN.xlsx EXP_SIGN!B44 `EXP(-745)`:
  Excel Online cached 0, ours 5E-324 (2^-1074). Whether that 0 is Excel's value or a saved kept subnormal is what the
  companion cells decide; the baseline line leaves eval/baseline/ironcalc-calc.txt if the engine then matches.

## Acceptance criteria
- [ ] A probes-numeric case with an excel-graph oracle covers both edges and `EXP(710)`, and every Excel quirk
  found is recorded in numbers.md.
- [ ] The engine matches every cell of the case, or each miss is a listed residual.
- [ ] `make check` green; reviewers approve.

## Out of scope
- EXP accuracy inside the range (T-1931, T-1940).

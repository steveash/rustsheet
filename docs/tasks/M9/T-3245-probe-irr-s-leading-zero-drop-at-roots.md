---
id: T-3245
title: "Probe IRR's leading-zero drop at roots far from 0.1 (T-3120 review item, left by T-3205)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence settles whether IRR drops leading zero flows (so `IRR({0,..,0,c...})` equals `IRR({c...})` from the
same guess) when the root is far from 0.1, where a residual check of 1e-7 scales by x^k; financial.md records the result.

## Context
- T-3120 drops leading zero flows on the evidence of three cells, all with a root near 0.1 (probes/irr-guess-edges
  A33, probes-numeric/m9-fin-irr-rate Calc!A173, A175). Its review (item 1, carried into T-3205) asked for a probe
  with far roots; T-3205 recorded it as unprobed (financial.md "IRR is a secant in d = 1 - v").
- Suggested cells: `{-1,0,..}`-style pairs with and without 1 to 3 leading zeros and roots near 5, 50, -0.5 and
  -0.9 (for example `IRR({0,-1,6})` against `IRR({-1,6})`, `IRR({0,0,-1,0.1})` against `IRR({-1,0.1})`), each at the
  default guess and at a guess near the root; write them with the excel-probe skill as a new probes case.
- Code: `compute_irr` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).

## Acceptance criteria
- [ ] A probes case with the cells above is filled by the oracle (excel-graph) and scored.
- [ ] financial.md says whether the drop holds at far roots, citing the cells; any mismatch is filed as an opus task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (a mismatch becomes its own task).

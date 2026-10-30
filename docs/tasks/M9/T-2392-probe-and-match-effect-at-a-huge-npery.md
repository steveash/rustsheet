---
id: T-2392
title: "Probe and match EFFECT at a huge npery (1E10): Excel's power differs in the eighth digit"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`EFFECT` at a huge npery matches Excel to 15 digits.

## Context
- Found by the T-2248 smoke check (one formula per financial edge-pool entry, excel-graph, 2026-10-22; the scratch
  suite was deleted, so the probe is this task's to commit). docs/excel-semantics/financial.md "Financial edge-pool
  smoke check (T-2248)" lists the observed values.
- Observed: `EFFECT(0.05,1E+10)` is 0.05127110072502061 in Excel, ours 0.05127110027547532 (a miss in the 9th digit);
  `EFFECT(0.05,1E+6)` = 0.0512710949426225 matches. financial.md "EFFECT and NOMINAL raise to their power with `^`":
  `(1+r/n)^n` by right-to-left square-and-multiply; at n = 1E10 the product chain may not be what Excel's `^` does
  (numbers.md "Integer exponents").
- Probe `(1+0.05/1E10)^1E10` with `^` directly and EFFECT at npery 1E7, 1E8, 1E9, 2^31-1, 2^31, 1E10 and 1E12 to find
  where Excel's integer power switches method. Code: `rate_power` in functions/financial/rate_conversions.rs.

## Acceptance criteria
- [ ] A probe case holds the sweep, filled through excel-graph.
- [ ] EFFECT (and `^` if it is the cause) matches every probed cell; numbers.md or financial.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- NOMINAL unless the same cause.

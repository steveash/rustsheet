---
id: T-0179
title: "Subnormal results become 0 (probes/precision-and-rounding A14: 2^-1074)"
milestone: M1
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A formula result below the smallest normal double (2.2250738585072014E-308) is 0, as in Excel: `=2^-1074` → 0 and `=CONCAT(1e-307/10)` → "0".

## Context
Rule VERIFIED in docs/excel-semantics/numbers.md ("Subnormal results become 0"). Failing: probes/precision-and-rounding A14 (engine gives 5e-324). No other task covers it: T-0110 (near-zero snapping) lists only A2–A4 and A15. Found by the 2026-10-06 steward planner pass. Look for the existing result-finalisation path (where overflow becomes #NUM!, T-0136) and flush subnormals in the same place; don't add a second one.

## Acceptance criteria
- [ ] probes/precision-and-rounding A14 passes and leaves the probes known failures; the probes-numeric `number-to-text/*/1e-307/10` cells do not regress; 0 eval regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

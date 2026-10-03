---
id: T-0900
title: "BESSELJ/BESSELY bit-exact parity"
milestone: M9
lane: functions
status: in-progress
priority: P2
size: M
depends_on: []
---

## Goal
Fix 145 failing BESSELJ/BESSELY cells in ironcalc-calc (dominant verdicts NumericClose/NumericMismatch).

## Context
`make eval-show CASE=ironcalc-calc/ENGINEERING/BESSEL.xlsx`. Current code derives from fdlibm (keep the Sun notice). Excel likely uses a different algorithm or argument truncation (n is truncated to integer).

## Acceptance criteria
- [ ] Known failures for BESSELJ/BESSELY removed from eval/baseline/ironcalc-calc.txt with no new ones.
- [ ] Findings recorded in docs/excel-semantics/engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

---
id: T-0900
title: "BESSELJ/BESSELY bit-exact parity"
milestone: M9
lane: functions
status: done
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

## Outcome
Excel uses Numerical Recipes rational approximations for `|x| < 8` (bit-exact now; 119 baseline
cells fixed, 1880 -> 1761). `|x| >= 8` still differs; split to T-0908. See
`docs/excel-semantics/engineering.md`.

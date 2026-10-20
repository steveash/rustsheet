---
id: T-2099
title: "IMEXP real part last digit: IMEXP(1+1.4E-8i) is 2.71828182845905 in Excel (T-2087 B9)"
milestone: M9
lane: functions-engineering
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`IMEXP("1+1.4E-8i")` matches Excel's real part to the last digit.

## Context
probes/eng-imexp-cutoff B9: Excel gives `2.71828182845905+3.80559455984266E-08i`, ours `2.71828182845904+...`. Ours is exp(1)*cos(1.4E-8) with cos = 1-2^-53 (the correctly rounded value); Excel's real part suggests its cos (or product) differs, or its exp is the x87 model (`excel_exp`, which alone did not fix it). Probe more small-y IMEXP values with a finite exponent and find the rule. `imexp` in complex.rs.

## Acceptance criteria
- [ ] `probes/eng-imexp-cutoff` B9 passes, with oracle-backed probes for the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

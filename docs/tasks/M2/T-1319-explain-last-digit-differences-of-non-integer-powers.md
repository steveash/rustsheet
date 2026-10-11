---
id: T-1319
title: "Explain last-digit differences of non-integer powers of 10 (10^33.5)"
milestone: M2
lane: functions-math
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Find the rule behind Excel's last-digit results for non-integer exponents of base 10 and implement it.

## Context
Excel `10^33.5` = 3.162277660168401E+33 and `10^-33.5` = 3.1622776601683576E-34 where libm `powf` gives ...379E+33 / ...3794E-34 (probes-numeric/power-large-integer-exponents-4.case.json A54-A56; numbers.md "Integer exponents beyond 360"). Code: `power` in crates/rustsheet-engine/src/arithmetic.rs. Probe more exponents (10^0.5, 10^2.5, 10^-2.5, 10^100.5) to find the rule.

## Acceptance criteria
- [ ] The three probe cells pass in `rustsheet-eval show` (probes-numeric is ungated and has no baseline); the rule is in numbers.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

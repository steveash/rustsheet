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

More cells (T-1275's gen-m7 suite, excel-graph, 2026-10-11; `./target/release/rustsheet-eval show gen-m7/<case>.case.json`):
- gen-m7/9ff3afcacd3bd0fc Calc!A13 `=(((7-Data!D2)^(0+Data!B12))/YEAR(MAX(1.005,Data!B9)))` = 6^2.5/1900: Excel 0.04641138460010231, engine 0.04641138460010232.
  Excel's quotient implies 6^2.5 = 88.18163074019438 (2 ulps below libm `powf`'s 88.18163074019441), which is exactly `exp(2.5*ln 6)`
  (also `sqrt(6)^5`). `exp(y*ln x)` also gives the probe's 10^33.5 and 10^-33.5, but not numbers.md's `2^(-0.5)` (Excel …475,
  exp-ln …476) or `3^-0.5` (Excel …258, exp-ln …257), so it is a lead, not the rule.

## Acceptance criteria
- [ ] The three probe cells pass in `rustsheet-eval show` (probes-numeric is ungated and has no baseline); the rule is in numbers.md.
- [ ] gen-m7/9ff3afcacd3bd0fc A13 passes, or the rule found explains why it cannot yet. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [ ] `make check` green; reviewers approve.

## Out of scope
-

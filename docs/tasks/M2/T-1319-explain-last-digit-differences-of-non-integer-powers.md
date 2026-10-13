---
id: T-1319
title: "Explain last-digit differences of non-integer powers of 10 (10^33.5)"
milestone: M2
lane: functions-math
status: done
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
- [x] The three probe cells pass in `rustsheet-eval show` (probes-numeric is ungated and has no baseline); the rule is in numbers.md.
- [x] gen-m7/9ff3afcacd3bd0fc A13 passes, or the rule found explains why it cannot yet. (T-1275's gen-m7 cells listed under Context, `rustsheet-eval show`.)
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- **Rule:** for a non-integer `y`, Excel computes `exp(y·ln x)`, a negative `y` as one over the positive power, and `y` = ±0.5 through `sqrt`. Recorded in numbers.md as "Non-integer exponents: exp(y·ln x)".
- **Evidence:** the new probes-numeric/power-non-integer-sweep (194 cells, excel-graph) has 193 of 194 cells passing. libm `pow` matches 93 of the 180 numeric cells in the main sweep.
- **Quirk found:** a negative exponent whose positive power underflows is #DIV/0!, integer exponents included; this is implemented.
- **Open:** `(-8)^(-1/3)` is -0.5 in Excel and -0.5000000000000001 in ours, and goes to T-0221. The three task cells pass, as does gen-m7/9ff3afcacd3bd0fc A13 (6^2.5).
- **Code:** `arithmetic::non_integer_power` replaces `powf`, including for odd roots of negatives. With it, POWER.xlsx `(-8)^(1/3)` = -1.9999999999999998, which is Excel's cached value.
- **Eval, 0 regressions in every suite:**
  - gen-m1 2 → 0 (T-1421's two cells, so gen-m1 is at 100%);
  - gen-m2 23 → 11;
  - ironcalc-calc 1478 → 1474;
  - probes 2797 → 2764;
  - poi unchanged.
- **Related tasks:** T-1421 is closed by this change. T-0221 keeps the remaining gen-m2 power cell (fa437879253827be A23).

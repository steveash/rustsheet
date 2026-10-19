---
id: T-1949
title: "Probe one-period YIELD when 30/360 DSR is 0 (settlement 30th, maturity 31st)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
One-period YIELD returns what Excel returns when the basis 0 or 4 DAYS360 settlement-to-maturity count (DSR) is 0 although settlement < maturity.

## Context
T-1532 made one-period YIELD a closed form ending in `* (f*E/DSR)` (`yield_closed.rs`). With settlement on the 30th and maturity on the 31st of one month DAYS360 gives DSR 0, the engine returns #NUM! (non-finite), and no oracle case shows Excel's result. Use the `excel-probe` skill.

## Acceptance criteria
- [ ] A probe case with Excel values for one-period YIELD at DSR 0 on bases 0 and 4; the engine matches it; the quirk is in docs/excel-semantics/financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

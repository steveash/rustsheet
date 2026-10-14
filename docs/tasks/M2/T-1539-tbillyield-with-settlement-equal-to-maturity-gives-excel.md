---
id: T-1539
title: "TBILLYIELD with settlement equal to maturity gives Excel's error, not a non-finite number"
milestone: M2
lane: functions-financial
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`TBILLYIELD(d,d,pr)` (zero days) gives what Excel gives, never an infinite or NaN number in a cell.

## Context
T-1475 security review: `fn_tbillyield` (crates/rustsheet-engine/src/functions/financial/mod.rs) divides
by `maturity - settlement`, so equal dates give `360/0` = inf and a non-finite result (pre-existing).
TBILLEQ maps non-finite results to `#DIV/0!`/`#NUM!`. Probe first (excel-probe): `TBILLYIELD(45366,45366,98)`,
`TBILLEQ(45366,45366,0.05)`, `TBILLPRICE(45366,45366,0.05)`, then match.

## Acceptance criteria
- [ ] The probe values are recorded in docs/excel-semantics/financial.md "TBILLEQ and TBILLYIELD" and pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other bond functions.

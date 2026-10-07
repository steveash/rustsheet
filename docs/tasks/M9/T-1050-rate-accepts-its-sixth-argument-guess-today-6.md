---
id: T-1050
title: "RATE accepts its sixth argument (guess); today 6 arguments give #ERROR!"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`RATE(nper,pmt,pv,fv,type,guess)` evaluates instead of returning `#ERROR!`, so the guess argument
reaches the solver as it does in Excel.

## Context
`fn_rate` (`crates/rustsheet-engine/src/functions/financial/mod.rs`) checks `!(3..=5).contains(&arg_count)`,
so the guess branch below it (`arg_count > 5`) can never run and every 6-argument call is an
argument-count error. Excel accepts 3 to 6 arguments (T-1009 sweep, `probes-numeric/m9-fin-irr-rate`:
the 42 `rate/.../guess=...` cells all evaluate in Excel; 30 are numbers and 12 are `#NUM!`).
See `docs/excel-semantics/financial.md`, "IRR, RATE and XIRR stop early".
Matching Excel's exact digits is T-0916's job (iteration fit), not this task's.

## Acceptance criteria
- [ ] `RATE` accepts 3 to 6 arguments. No `rate/.../guess=` cell in `probes-numeric/m9-fin-irr-rate`
      shows `#ERROR!` (`make eval-show CASE=probes-numeric/m9-fin-irr-rate.case.json`). Any that stay
      failing differ only in value or as `#NUM!`/number, and are left for T-0916.
- [ ] A unit test covers a 6-argument RATE call, and a 7-argument call stays an argument-count error.
- [ ] No regressions: `make eval-baseline` refuses none, and the baseline only shrinks.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Matching Excel's iteration, tolerance or guess-dependent digits (T-0916).

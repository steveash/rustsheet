---
id: T-3585
title: "RATE is #NUM! in Excel where the engine finds a root: type-1 annuities at a high rate, huge-pv annuities near rate 0"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: S
tier: opus
depends_on: [T-3535]
---

## Goal
RATE returns `#NUM!` where Excel does on the seven Gate A points below, and financial.md records the rule that
predicts them.

## Context
- Found by T-3535's first RATE table (accuracy/RATE/s1, excel-graph). Excel gives `#NUM!` and the math has a root at
  every point; R3 never excuses an error mismatch, so Excel's error is the target (directive 1).
  - **Type-1 annuities at a high rate, fv 0.** The engine returns a bogus root near -1 (-0.9999999999999…), neither the
    math nor Excel's error:
    - A19 `RATE(77,-2757.6011275919,18790.0791146529,0,1)`, root 0.172;
    - A128 `RATE(230,6189.01060624196,-9436.11470677501,0,1)`, root 1.91;
    - A129 `RATE(329,613.284600034776,-815.966938950879,0,1)`, root 3.03;
    - A137 `RATE(280,92693.8285831394,-125054.343644821,0,1)`, root 2.86;
    - A158 `RATE(257,-53538.3890651085,83315.7312547417,0,1)`, root 1.80.
  - **Huge pv, small fv, root near 0.** The engine returns the math:
    - A278 `RATE(29,622792052.26538,-18060969526.6562,179.964221458841,0)`, root 6.24E-10;
    - A291 `RATE(43,-1768871036.99479,76057734597.4655,-42.6330901500337,0)`, root 2.22E-6.
- They are carried in `eval/accuracy/known-failures.tsv` with this task as owner.
- Prior work: financial.md "IRR, RATE and XIRR stop early" and "RATE is a secant on the FV residual" (T-2005, T-0916,
  T-1525 probed the iteration). T-3348 fixed the opposite case (Excel's value where the engine gave `#NUM!`).
- Code: `fn_rate` in `crates/rustsheet-engine/src/functions/financial/annuity.rs`.

## Acceptance criteria
- [ ] A probe brackets each group's `#NUM!` (rate, type and nper for the first; pv/pmt scale and fv for the second), and
      financial.md records the rule with evidence.
- [ ] The engine returns `#NUM!` on the probed cells and the seven points; their known-failures lines are removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- RATE's accuracy where Excel returns a number.

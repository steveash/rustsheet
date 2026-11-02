---
id: T-3585
title: "RATE is #NUM! in Excel where the engine finds a root: type-1 annuities at a high rate, huge-pv annuities near rate 0"
milestone: M9
lane: functions-financial
status: done
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
  - **Huge pv, small fv, root near 0:**
    - A278 `RATE(29,622792052.26538,-18060969526.6562,179.964221458841,0)`, root 6.24E-10. The engine returns
      -9.45E-10, a root of the wrong sign: neither the math nor Excel's error.
    - A291 `RATE(43,-1768871036.99479,76057734597.4655,-42.6330901500337,0)`, root 2.22E-6. The engine is near the
      math (1.7E-8 relative).
- They are carried in `eval/accuracy/known-failures.tsv` with this task as owner.
- Prior work: financial.md "IRR, RATE and XIRR stop early" and "RATE is a secant on the FV residual" (T-2005, T-0916,
  T-1525 probed the iteration). T-3348 fixed the opposite case (Excel's value where the engine gave `#NUM!`).
- Code: `fn_rate` in `crates/rustsheet-engine/src/functions/financial/annuity.rs`.

## Acceptance criteria
- [x] A probe brackets each group's `#NUM!` (rate, type and nper for the first; pv/pmt scale and fv for the second, and
      the sign of the engine's root there), and financial.md records the rule with evidence.
- [x] The engine (group 1; group 2 moved to T-3424, see Outcome) returns `#NUM!` on the probed cells and the seven points; their known-failures lines are removed.
- [x] `make check` green; reviewers approve.

## Out of scope
- RATE's accuracy where Excel returns a number.

## Outcome
- Probe probes/t3585-rate-num-near-minus-one (104 cells, excel-graph 2026-11-01). Group 1: Excel returns a converged
  root only when 1 + r is above 1e-7 (bracket 1E-7 .. 1.0000034E-7), for type 0 and type 1, genuine roots near -1
  and the spurious root -1 of a type-1, fv-0 residual alike. `compute_rate` now checks `1 + r > 1e-7` (the step bound)
  at convergence. financial.md "RATE is #NUM! when 1 + r is 1e-7 or less" replaces "RATE is #NUM! at the spurious
  root r = -1".
- Group 2 (A278, A291) is not a rule on scale or fv: Excel's error comes and goes with the scale (A291 errs at 2^0
  and 2^-6 but not 2^-2 or 2^-4) and the engine ends both in a flat-secant hold in the residual's noise. They are
  T-3424's kind; their known-failures lines now name T-3424 and T-3424's context lists them.
- Eval delta: Gate A RATE known failures 7 -> 2 (owner T-3424); the new probe 97/104 (the 7 off are its G/H noise
  cells, baselined); probes-numeric/t3478-rate-r4-wide 286 -> 299/300, which also settles T-3547 (marked done).
  No regressions (`make eval-baseline` only added the new case).
- Not a numeric fit: the floor reuses the documented 1e-7 step bound; it lies in the probed bracket.

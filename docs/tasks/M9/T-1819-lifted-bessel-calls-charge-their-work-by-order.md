---
id: T-1819
title: "Lifted BESSEL calls charge their work by order, not a flat weight"
milestone: M9
lane: functions-engineering
status: done
priority: P2
size: M
tier: opus
depends_on: [T-1383]
---

## Goal
A lifted BESSELI/J/K/Y call is refused only when its total recurrence work is too large, not
whenever it has more than 2,048 elements: `BESSELJ(SEQUENCE(5000)/100,0)` and
`ROWS(BESSELJ(2.5,SEQUENCE(2100)))` return Excel's values, while a lift whose orders sum past the
work budget is still `#ERROR!` before any element runs.

## Context
T-1383 made the Bessel functions `KernelWork::Weighted(2^16)` (functions/lift/kinds.rs `BESSEL`),
so `MAX_LIFTED_CELL_VISITS` (2^27) caps every lifted call at 2,048 elements whatever the orders.
The kernels' cost is O(n) in the order n, not in x: `excel_bessel_j` n = 0/1 is O(1); the
recurrences, fdlibm `jn`/`yn`, `bessel_i` and `bessel_k` are about n to 2n steps, capped by
`transcendental::MAX_ORDER` (100,000). The flat weight is the docs/excel-semantics/engineering.md
"BESSEL order cap" DEVIATION and is pinned by test/engineering/test_argument_lift.rs
`bessel_work_is_bounded`. The performance review of T-1383 proposed a value-dependent kind, e.g.
`KernelWork::ByArg { arg, base, per_unit }`, summed over the broadcast elements in `lift_grids`
before anything runs (O(elements), or elements × f(n) for a scalar n), with `MAX_ORDER` kept as the
per-call cap. Measured worst case: about 1 ms per element at n = 100,000 (release, BESSELI).
This changes the SPEC-0011 driver: amend the spec's "Security rule for kernel work" in the same PR.

## Acceptance criteria
- [x] The two calls in the Goal give Excel's values (scalar controls / a probe), and a lift whose
  summed order work exceeds the budget is `#ERROR!` without running any element (test).
- [x] The worst case stays bounded to about 2 s in release (measure it and note the figure).
- [x] The engineering.md DEVIATION entry and SPEC-0011 describe the narrowed cap.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other value-dependent kernels; this task only adds the work kind and uses it for BESSEL*.

## Outcome
New `KernelWork::ByArg { arg, cap, weight }` (lift/kinds/mod.rs). Before any element runs, the
driver (`lift/driver.rs` `kernel_units`) charges each element by its order:
`1 + min(trunc(n), cap) * weight / cap` for a number, the cap for a text, and 1 otherwise. That is
one O(elements) pass. BESSELI/J/K/Y use `arg: 1, cap: MAX_ORDER, weight: 2^16 - 1`. An element at
the cap still costs 2^16, so at most 2,048 run, as before. Small orders now lift freely:
`ROWS(BESSELJ(2.5,SEQUENCE(2100)))` = 2100 and `ROWS(BESSELJ(SEQUENCE(5000)/100,0))` = 5000.
`ROWS(BESSELI(2.5,SEQUENCE(2100,1,100000,0)))` is `#ERROR!` before any element runs.
Worst case in release, 2,048 elements at the cap: BESSELI 1.88 s, BESSELJ 0.73 s, BESSELK 0.52 s,
BESSELY 0.51 s. SPEC-0011's security rule and the engineering.md "BESSEL order cap" DEVIATION
describe the narrowed cap. Eval delta: none; no suite cell lifts a Bessel call past 2,048 elements.

---
id: T-2105
title: "Move the double-double primitives (two_sum, fast_two_sum, two_prod, dd_*) out of x87_exp into a double_double module"
milestone: M9
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-1945]
---

## Goal
The general error-free transforms and double-double helpers have one home, a small `double_double` module in the engine core, instead of living in `x87_exp` (named for one function) and being imported from there by trig.rs.

## Context
- Raised by the architecture and quality reviews of T-1945: `x87_exp.rs` holds `two_sum`, `fast_two_sum`, `two_prod` (Dekker), `dd_mul`, `dd_add`, `dd_div`, `dd_div_small` and `dd_to_extended`, and trig.rs now calls `x87_exp::two_sum` and `fast_two_sum`. trig.rs's `DoubleDouble` also keeps a `mul_add` `two_prod`.
- Pure move: no behavior change, bits identical.

## Acceptance criteria
- [ ] The helpers live in `crates/rustsheet-engine/src/double_double.rs` (or a similar core module); `x87_exp`, `erf` and trig.rs import them from there, and nothing is duplicated (`scripts/check_duplication.py` does not grow).
- [ ] The `x87_exp` tests and probes-numeric `t1931-exp-*` and `power-*` keep their results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing any numeric code; merging trig.rs's `DoubleDouble` struct into the free functions.

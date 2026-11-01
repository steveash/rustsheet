---
id: T-3459
title: "Charge the half-integer upper gamma's backward fraction in the incomplete-gamma lift weight"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3285]
---

## Goal
The lifting table's charge for an upper incomplete-gamma cdf covers the work T-3285's accurate half-integer kernel
adds, so a lifted CHISQ.DIST.RT / CHIDIST / CHISQ.TEST at an odd df from 3 to 39 is not under-charged (directive 5).

## Context
- T-3285 (`statistical/incomplete_gamma/half_integer.rs`, `fraction.rs`): for a half-integer shape a in [3/2, 20) and
  a + 1 <= x <= 650, CHISQ.DIST.RT's Q first runs the forward (Lentz) fraction to find its depth n, then the same
  fraction backward in double-double from depth 2n + 8, then a double-double prefactor (at most 19 factors); where the
  result is not a normal double it falls back to the ordinary pair, which runs the forward fraction again.
- `crates/rustsheet-engine/src/functions/lift/kinds/mod.rs` `incomplete_gamma_units` charges a cdf by
  `cdf_terms_bound(a)` terms at `INCOMPLETE_GAMMA_TERM_COST` (4.7 ns, the forward fraction's NaN path, T-1872) plus
  `INCOMPLETE_GAMMA_CDF_COST`. T-3285's security review estimated that this under-counts the backward fraction by
  about 2 to 3 times (each backward step is a `dd_add` and a `dd_div`, and there are about twice as many as forward
  steps). The work stays bounded: a < 20 gives a forward budget of at most 256 + 32 cbrt(20) = 343 terms, so at most
  694 backward steps per cdf, independent of x; only the charge's proportionality is off.
- Measure the backward fraction's cost per step (release build, as T-1872 measured the term cost) and fold it into
  the charge for the half-integer shapes the kernel takes (or into the term cost for every shape, if simpler and
  within the existing weight), keeping the charge a bound from the arguments' values alone.

## Acceptance criteria
- [ ] A measured per-step cost of the backward fraction recorded beside `INCOMPLETE_GAMMA_TERM_COST`, and a charge
      that bounds the accurate kernel's work (forward + backward + fallback) for a in [3/2, 20).
- [ ] A unit test in lift/kinds that the charge at a half-integer shape is at least the measured work's bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the kernel's values (T-3285, T-3269).

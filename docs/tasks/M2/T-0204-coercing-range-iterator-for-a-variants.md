---
id: T-0204
title: "Shared coercing range iterator for the *A variants and GCD/LCM"
milestone: M2
lane: functions
status: done
priority: P2
size: S
depends_on: [T-0200]
---

## Goal
Replace the remaining hand-written `clip_to_used_area` walks in VARA/STDEVA (variance.rs,
standard_dev.rs) and GCD/LCM with a counterpart of `Model::for_each_number_in_range` that
coerces text and booleans in references, so every aggregate range walk lives in one place.

## Context
Left out of T-0200 because their coercion differs. Consider moving both helpers to a dedicated
`functions/args.rs` module (util.rs is becoming a grab-bag).

## Acceptance criteria
- [ ] One coercing helper replaces the remaining walks in variance.rs, standard_dev.rs, gcd_lcm.rs.
- [ ] Duplicated lines reduced; eval unchanged or improved; `make check` green.

## Out of scope
Changing semantics.

## Outcome
VARA/VARPA/STDEVA/STDEVPA and GCD/LCM range walks now use `for_each_coerced_number_in_range` / `for_each_cell_in_range` in `functions/util.rs`. No semantic change. Module split into `args.rs` left out (not needed).

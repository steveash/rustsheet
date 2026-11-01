---
id: T-3458
title: "One pub(crate) Dd double-double alias next to the dd_* helpers in x87_exp.rs"
milestone: M9
lane: quality
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-3285]
---

## Goal
One definition of the double-double pair type: a `pub(crate) type Dd = (f64, f64);` beside the `dd_*` helpers and
`two_sum` in `crates/rustsheet-engine/src/x87_exp.rs`, used by every module that now declares its own.

## Context
- T-3285's quality review (CLAUDE.md prime directive 3, one definition per concept): five private aliases
  `type Dd = (f64, f64)` exist, in `statistical/incomplete_gamma/fraction.rs` (`pub(super)`, also used by
  `incomplete_gamma/half_integer.rs`), `statistical/loader.rs`, `statistical/bratio/bgrat.rs`,
  `statistical/discrete.rs` and `statistical/student_t.rs`. The `dd_*` helpers in x87_exp.rs take and return
  `(f64, f64)` directly.
- `half_integer_beta.rs` spells its double-doubles `(f64, f64)` (`PI_DD`, `FRAC_1_SQRT_PI`); they can take the alias.
- Pure refactor: no value changes. `rg "type Dd"` must find one line afterwards.

## Acceptance criteria
- [ ] `rg -n "type Dd" crates/` finds only the x87_exp.rs definition (with a doc comment); the five modules import it.
- [ ] The `dd_*` helpers' signatures may use the alias; no behaviour change (`make eval-check` unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the helpers' algorithms or moving them out of x87_exp.rs.

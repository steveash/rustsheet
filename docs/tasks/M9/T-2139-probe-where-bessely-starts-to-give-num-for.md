---
id: T-2139
title: "Probe where BESSELY starts to give #NUM! for large orders and Excel's BESSELJ order cap"
milestone: M9
lane: research
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence for two unprobed BESSEL order limits: the first order n where `BESSELY(x,n)` is `#NUM!`, and whether `BESSELJ` has any order cap. Both are recorded in `docs/excel-semantics/engineering.md`.

## Context
Found by T-0908's review. probes-numeric/m9-bessel (T-1004) shows `BESSELY(10,101)` finite and `#NUM!` at n = 1000, but nothing in between. It also shows `BESSELJ(x,1000000)` = 0, while RustSheet caps every order at 100,000 (`MAX_ORDER` in
`functions/engineering/transcendental/bessel_excel.rs`; "BESSEL order cap" in engineering.md is a DEVIATION).
Does Y's `#NUM!` come from overflow of the upward recurrence, giving +-inf, or from an explicit cap? Bisect n in 101..1000 at x = 1, 10 and 50.

## Acceptance criteria
- [ ] A probes case (tag M9) that bisects the first `#NUM!` order of BESSELY at x = 1, 10, 50, oracle-filled.
- [ ] A few BESSELJ cells at orders 100001..2^31-1 (e.g. `BESSELJ(10,2^31-1)`) oracle-filled. engineering.md states what Excel does there.
- [ ] Any engine mismatch is filed as a follow-up task or fixed. The order cap stays unless it needs a decision record.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Raising or removing `MAX_ORDER` (directive 5).

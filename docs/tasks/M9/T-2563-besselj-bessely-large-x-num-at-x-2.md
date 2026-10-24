---
id: T-2563
title: "BESSELJ/BESSELY large x: #NUM! at x = 2^27 (orders 0, 3) and 1e10, last-digit drift at 1e3..1e6"
milestone: M9
lane: functions-engineering
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BESSELJ/BESSELY match Excel above x = 100: `#NUM!` where Excel returns it and the same last digits at 1e3..1e6.

## Context
Probe case `bessel-large-x` (T-2140, 24 cells, 21 failing) and docs/excel-semantics/engineering.md. Excel returns `#NUM!` at x = 2^27 (orders 0, 3) and 1e10 (all orders); the engine returns numbers. x = 1e3..1e6 differ by 1 ulp to ~1e-14 relative (phase computation). Code: `functions/engineering/transcendental/bessel_excel.rs` (`asymptotic`).

## Acceptance criteria
- [ ] Every cell of `bessel-large-x` passes (or is a documented DEVIATION), baseline updated.
- [ ] The pattern of which (x, order) pairs give `#NUM!` is explained in engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other Bessel functions (BESSELI, BESSELK).

---
id: T-2563
title: "BESSELJ/BESSELY large x: #NUM! at x = 2^27 (orders 0, 3) and 1e10, last-digit drift at 1e3..1e6"
milestone: M9
lane: functions-engineering
status: done
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
- [x] Every cell of `bessel-large-x` passes (or is a documented DEVIATION), baseline updated.
- [x] The pattern of which (x, order) pairs give `#NUM!` is explained in engineering.md.
- [x] `make check` green; reviewers approve (in T-2547's PR).

## Out of scope
- Other Bessel functions (BESSELI, BESSELK).

## Outcome
Done in T-2547's PR: the asymptotic form's phase past 134217727 is `#NUM!` and its sine and cosine are Excel's
(`excel_sin_cos`), so all 24 cells of bessel-large-x pass bit for bit; the pattern is in engineering.md "BESSELJ / BESSELY"
(probes/t2547-bessel-phase-cut).

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

Also (T-1741, decision 0107): `=LINEST(EVEN(BESSELJ(1000000000000000.0,1)))`, gen-m9/c468f5046aadfbbc Calc!A122 and
its spill cell B122: Excel's BESSELJ(1e15,1) is `#NUM!`, so the anchor is `#NUM!` and nothing spills; ours computes a
value, so LINEST of the one point spills `{0,2}`.

## Acceptance criteria
- [ ] gen-m9/c468f5046aadfbbc Calc!A122 and B122 pass; their lines leave eval/baseline/gen-m9.txt.
- [ ] Every cell of `bessel-large-x` passes (or is a documented DEVIATION), baseline updated.
- [ ] The pattern of which (x, order) pairs give `#NUM!` is explained in engineering.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other Bessel functions (BESSELI, BESSELK).

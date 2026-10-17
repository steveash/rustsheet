---
id: T-1800
title: "The lifting driver takes error precedence from the kernel's own check order, with a refused range as a #VALUE! at its position (probes/scalar-lift-open-a, -d)"
milestone: M9
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
An element's error follows the kernel's own scalar check order, and a multi-cell range refused in a `LiftArrays` argument is a `#VALUE!` argument at its position in that order, not an error picked by argument position before the kernel runs.

## Context
SPEC-0011 "Errors and element values" assumed the leftmost failing argument wins, as MROUND does today. T-1777 (probes/scalar-lift-open-a W1-W17, -d W1-W16, docs/excel-semantics/arrays.md) measured it: for EDATE, EFFECT, ACCRINTM and PMT the leftmost wins (`EDATE(NA(),A1:A60)` is `#N/A`, `EDATE(A1:A60,NA())` is `#VALUE!`; `INDEX(PMT({#NUM!},{#N/A},100),1)` is `#NUM!`, reversed `#N/A`). YEARFRAC checks `basis` first, then the start date, then the end date, in the scalar call too (`YEARFRAC(1/0,45000,NA())` is `#N/A`, -d W1), so `INDEX(YEARFRAC(1/0,{45000;45031},B1:B60),1)` is `#VALUE!` (the range at `basis`, -a W6) and `INDEX(YEARFRAC({45000;45031},B1:B60,NA()),1)` is `#N/A` (-a W7). A driver that picks the leftmost error before calling the kernel gives `#DIV/0!` for W6 and `#VALUE!` for W7. So the driver hands the refused range to each element's kernel call as a `#VALUE!` in the range's slot, and the kernel's own order decides. T-1770 (driver; ours today: YEARFRAC's scalar kernel checks start, end, basis at yearfrac.rs:22-37, so -d W1, W3, W4, W6, W7 fail before any lifting), T-1372 and T-1383 (ATP kernels with an order of their own) must account for it.

## Acceptance criteria
- [ ] The probes/scalar-lift-open-a cells W1-W17 and probes/scalar-lift-open-d W1-W16 pass.
- [ ] YEARFRAC's scalar kernel checks basis, start, end (crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs reads start, end, basis today), so -d W1, W3, W4, W6, W7 pass. This part does not need T-1770 and can land first.
- [ ] The driver never scans the arguments for the first error: a refused range reaches the kernel as `#VALUE!` in its slot, pinned by -a W1-W17 and -d W8-W16 (the second part, with T-1770).
- [ ] A unit test pins YEARFRAC's order (basis, start, end) with an error and a refused range in two of the slots.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reordering kernels other than YEARFRAC (their order matched Excel in the probed cells).

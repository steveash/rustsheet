---
id: T-1800
title: "The lifting driver takes error precedence from the kernel's own check order, with a refused range as a #VALUE! at its position (probes/scalar-lift-open-a, -d)"
milestone: M9
lane: engine
status: done
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
- [x] YEARFRAC's scalar kernel checks basis, start, end (crates/rustsheet-engine/src/functions/date_and_time/yearfrac.rs reads start, end, basis today), so -d W1, W3, W4, W6, W7 pass. This part does not need T-1770 and can land first.
- [x] The driver never scans the arguments for the first error: a refused range reaches the kernel as `#VALUE!` in its slot, pinned by -a W1-W17 and -d W8-W16 (the second part, with T-1770).
- [x] A unit test pins YEARFRAC's order (basis, start, end) with an error and a refused range in two of the slots.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reordering kernels other than YEARFRAC (their order matched Excel in the probed cells).

## Outcome
YEARFRAC's kernel (`yearfrac_of`, functions/date_and_time/yearfrac.rs) now reads the basis, range-checks it (0-4, else `#NUM!`), then reads the start and the end date, as Excel does. The driver already ran no error pre-scan (T-1770 hands a refused range to the kernel as `#VALUE!` in its slot); this task pins that and changes no other kernel.

- **Evidence**: probes/scalar-lift-open-d W1-W7, -a W5-W8, and a new probe, probes/yearfrac-basis-order (14 cells, excel-graph 2026-10-17): a number basis out of range is `#NUM!` before either date is read (`YEARFRAC(NA(),45000,5)`, `YEARFRAC("x",45000,5)`, `YEARFRAC(TRUE,45000,5)`, `YEARFRAC(NA(),45000,-1)` are `#NUM!`), a non-number basis wins with its own error (`YEARFRAC(NA(),45000,"x")` and `(..,TRUE)` are `#VALUE!`), numeric text is a basis. Recorded in dates.md "Date function argument modes" and arrays.md.
- **Callers of `get_yearfrac`** (ACCRINTM, DISC, INTRATE, PRICEDISC, PRICEMAT, YIELDDISC, YIELDMAT, RECEIVED) pass three numbers; every check either order can fail on is `#NUM!`, so nothing changes for them (no eval cell moved).
- **Tests** (functions/lift/tests/mod.rs): `yearfrac_checks_basis_then_start_then_end` (the -d W1-W7, -a W5-W8 and yearfrac-basis-order forms, errors and refused ranges in two slots), `the_driver_hands_a_refused_range_to_the_kernel_in_its_slot` (counting kernels returning one slot: EDATE with `NA()` beside a refused range in either order, one call each; YEARFRAC with `1/0` and a refused basis beside an array, both slots seen in every element).
- **Not met here**: -a W4 (`EFFECT(C1:C60,1/0)`) and -d W9, W11, W13, W14, W16 (ACCRINTM over a range/array) still fail because EFFECT and ACCRINTM do not lift yet (`#N/IMPL!`, or a range intersected); that is T-1372's lifting, and their order is already leftmost as Excel's is. The first acceptance box stays open for T-1372.
- **Eval delta**: `make eval-baseline` (`--timeout 240`, loaded box): 0 regressions, probes known failures 4684 -> 4677, fixed scalar-lift-open-a W6, W7 and -d W1, W3, W4, W6, W7; new case yearfrac-basis-order passes 14/14. Every suite, the non-gated ones too (gen-m5..gen-m9, probes-numeric), compared cell by cell against main at `--timeout 240` (values of failing cells included): 0 new failing cells, 0 changed failing values, 17 cells fixed (the 7 above and 10 of yearfrac-basis-order, which main fails); the same 5 corpus-bench cases time out on both sides (already baselined).


---
id: T-1821
title: "NPV's rate, IRR's guess and MIRR's rates lift over arrays and ranges beside their whole cash flows (SPEC-0011)"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: S
tier: opus
depends_on: [T-1372, T-1772]
---

## Goal
NPV's rate, IRR's guess and MIRR's finance and reinvest rates lift over arrays and multi-cell ranges, with the cash
flows read once, whole, and handed to every element.

## Context
probes/fin-argument-modes: `ROWS(NPV(G59:G60,K1:K5))`, `ROWS(IRR(K1:K5,G47:G48))` and `ROWS(MIRR(K1:K5,G53:G54,0.12))`
are 2 in Excel, and so is each `{v;v}` cell of those arguments (financial.md "Financial function argument modes").
NPV's values, IRR's and MIRR's values are `WholeByCaller` (SPEC-0011; their read rules are T-1373's): the caller reads
the cash flows once, the kernel captures the result (an error in the flows is every element's error, after the
elements' own earlier arguments in the scalar read order). IRR's solver evaluates NPV at most a few hundred times per
call over n flows, so its entry needs a cost of `LiftCost::Cells(n)` with a `KernelWork::Weighted` step weight
(SPEC-0011 security rule); `lift_call_costed` (T-1772) is the entry for a costed call. T-1372 left these out while T-1772 landed
the first `WholeByCaller` consumer (LARGE, SMALL, PERCENTILE, ...); follow its pattern.

## Acceptance criteria
- [x] The NPV-rate, IRR-guess and MIRR-rate lift cells of probes/fin-argument-modes pass (baseline lines removed).
- [x] A test shows the cash flows are evaluated once for a lifted call, and that a lifted IRR over large flows is
  refused by the work bound before the first element.
- [x] `make check` green; reviewers approve.

## Out of scope
- How the cash flows are read (T-1373), solver parity (T-0916).

## Outcome
NPV, IRR and MIRR are in the lifting table (`kinds/financial.rs`): NPV `[Lift]` then repeated `WholeByCaller`, IRR
`[WholeByCaller, Lift]` weighted `IRR_WEIGHT` (512, the derived bound of 455 NPV passes per call), MIRR
`[WholeByCaller, Lift, Lift]`. Each caller reads its cash flows once, before the driver, and calls
`lift_call_costed` with `LiftCost::Cells(flows)`; an error in the flows is every element's, after NPV's rate and
before IRR's guess and MIRR's rates (the scalar read order). Their hand-written static-analysis arms are gone
(the table derives them). Tests: `test_fn_financial_lift.rs` (lift shapes and element values, error order, flows
evaluated once via RANDARRAY, the work bound refusing `IRR(A1:A300000,{0.1;0.1})` before the first element).
Eval: probes known failures 1684 -> 1677 (fin-argument-modes H155, H156, H172-H174, H187, H188), no regressions.

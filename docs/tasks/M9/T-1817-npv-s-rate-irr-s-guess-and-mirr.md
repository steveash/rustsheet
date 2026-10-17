---
id: T-1817
title: "NPV's rate, IRR's guess and MIRR's rates lift over arrays and ranges beside their whole cash flows (SPEC-0011)"
milestone: M9
lane: functions-financial
status: todo
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
- [ ] The NPV-rate, IRR-guess and MIRR-rate lift cells of probes/fin-argument-modes pass (baseline lines removed).
- [ ] A test shows the cash flows are evaluated once for a lifted call, and that a lifted IRR over large flows is
  refused by the work bound before the first element.
- [ ] `make check` green; reviewers approve.

## Out of scope
- How the cash flows are read (T-1373), solver parity (T-0916).

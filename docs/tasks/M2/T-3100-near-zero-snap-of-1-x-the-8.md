---
id: T-3100
title: "Near-zero snap of 1-X: the 8-ULP bound follows the left operand, not the smaller addend"
milestone: M2
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Excel's bound for the near-zero snap of a root `+`/`-` is established by probe and implemented, including the case
where the operand order decides it.

## Context
- probes/t3073-lookup-near-equal-approximate Sheet1!E8 `=V!C29-V!A30`, which is `1-(1-8*2^-53)`, is 0 in Excel. The
  engine keeps 8.881784197001252e-16. E16, `(1-8*2^-53)-1`, keeps -8.881784197001252e-16 in both.
  docs/excel-semantics/numbers.md "Near-zero snapping" says Excel snaps iff |result| < 8 ULP of the smaller addend.
  For both cells that bound is 2^-50, the same as the result, so neither should snap.
- Both cells fit a bound of 8 ULP of the left operand (`1` gives 2^-49, `1-8*2^-53` gives 2^-50). So does every
  earlier probe, where X was always the left operand (snap-addsub, `X-b`). That is one cell, so probe before
  changing anything:
  - `b-X` and `-X+b` for X = b·(1±k·2^-52), k = 3 to 8, over a few bases;
  - `X+(-b)` and `-b+X`;
  - SUM with the addends in both orders.
- Code: `snap_near_zero` (arithmetic.rs).

## Acceptance criteria
- [x] An excel-graph probe case (milestone M2) with the cells above. numbers.md records the rule that fits every cell
      of it and of snap-addsub.
- [x] t3073-lookup-near-equal-approximate Sheet1!E8 and the new cells pass. `make eval-baseline` is run, with no regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- The lookup comparator (T-3099).

## Outcome
- Probe `probes/t3100-snap-operand-order` (excel-graph, 540 cells): b = 1, 2, 1.5, 3, 0.75, X = b±k·2^(e-52), k = 3..8,
  in `X-b`, `b-X`, `-X+b`, `X+(-b)`, `-b+X` and SUM with the addends in both orders. Every cell fits a bound of 8 ULP
  of the **left** addend (SUM: the total before its last addend); the smaller-addend rule misses 32 cells and a
  larger-addend rule 40. numbers.md "Near-zero snapping" states the rule (snap-addsub, where X was always on the left,
  fits it too).
- `snap_near_zero` (arithmetic.rs) takes its scale from the left addend `x`; every caller already passes the left
  operand (or SUM's running total) first. The hand-written unit expectation `1-(1-4·2^-52)` kept was wrong by this
  evidence and now snaps.
- Eval delta: probes fixed 1 (t3073-lookup-near-equal-approximate Sheet1!E8), new case 540/540, regressions 0 in every
  suite.

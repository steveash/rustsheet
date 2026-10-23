---
id: T-2241
title: "Probe EFFECT overflow error and the exact-power-of-ten base (EFFECT(1E300,2), EFFECT(270,30))"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
campaign: visible
---

## Goal
docs/excel-semantics/financial.md "EFFECT and NOMINAL raise to their power with `^`" records Excel's value for the two
edge cases that T-2034 left unprobed. The engine then matches Excel on both.

## Context
T-2034 moved EFFECT and NOMINAL onto `arithmetic::power` (Excel's `^`). The Excel-fidelity review raised two
behaviours that no oracle cell covers:
- Overflow: `rate_power` (functions/financial/rate_conversions.rs) maps an overflowing EFFECT to `#DIV/0!`, as the old
  infinite-rate path did. Excel's `^` itself is `#NUM!` on overflow. Excel's EFFECT could be either.
- Exact powers of ten: `power` returns 1E{n} exactly for a base of exactly 10 and n from 23 to 308
  (`decimal_power_of_ten`), so `EFFECT(270,30)` is now 1E30 - 1. Excel may instead square-and-multiply there.
Write a probes case per the `excel-probe` skill. Suggested cells: `EFFECT(1E300,2)`, `EFFECT(1E200,3)`,
`EFFECT(1E10,100)`, `EFFECT(270,30)`, `EFFECT(9,1)`, `EFFECT(18,2)`, `EFFECT(990,99)`, `NOMINAL(1E300,0.5)`. Get the
oracle values and record them in the entry.

T-2257 sweep (gen-m9 financial; gap triage 2026-10-23): EFFECT with a huge nominal rate is `#NUM!` in Excel, not ours `#DIV/0!`: `EFFECT(1E+300,11)`, `EFFECT(1E+300,3)`, `EFFECT(123456789.123,365)`, `EFFECT(1E13,365)`, `EFFECT(1E15,1E15)`. This is the overflow answer the probe was to find.

Owned gen-m9 cells (T-2257 triage, excel-graph, 2026-10-23): 8 known-failure lines in eval/baseline/gen-m9.txt, 8 visibly off.
Examples:
- gen-m9/3c06f9298f65a3ba Calc!A254 `=EFFECT(1E+300,6)`: Excel #NUM!, ours #DIV/0!
Cells:
- gen-m9/31474c20fc1dafd6.case.json (1): Calc!A37
- gen-m9/3c06f9298f65a3ba.case.json (1): Calc!A254
- gen-m9/5c6a23c4f6597f1c.case.json (2): Calc!A182, A378
- gen-m9/7d2979fba2c95d64.case.json (1): Calc!A266
- gen-m9/89f0aee953a55a45.case.json (1): Calc!A78
- gen-m9/a91bad4e84f49739.case.json (1): Calc!A155
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A5

## Acceptance criteria
- [ ] Visible parity (gen-m9, T-2257): the 8 visibly-off known-failure lines listed under "T-2257 sweep" in Context disappear
      from eval/baseline/gen-m9.txt (`make eval-baseline`), with no new known-failure lines in any baseline.
- [ ] A probes case with the cells above has excel-graph values, and its known failures, if any, are in the baseline.
- [ ] The financial.md entry states Excel's overflow error and its exact-power-of-ten behaviour, citing the cells, and
      drops "Not probed".
- [ ] If the engine differs, `rate_power` is changed to match (a one-line error mapping or a guard), with a unit test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other financial functions' overflow errors.

## Visible-parity stage
Decision 0083 (gap triage 2026-10-23, T-2257): the gen-m9 cells listed under "T-2257 sweep" in Context are visibly off (wrong
type, error, text, or a value off by more than half a unit of the 15th significant digit), so the task is in the campaign. Its
target is that every one of them has Excel's type, error and text, or is within half a unit of the 15th significant digit of
Excel's value; the T-2257 visible-parity criterion under Acceptance criteria states it. Bit-level criteria still unmet then move
to a follow-up task filed without the key.

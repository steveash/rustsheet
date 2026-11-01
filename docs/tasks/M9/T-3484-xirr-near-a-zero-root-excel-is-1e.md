---
id: T-3484
title: "XIRR near a zero root: Excel is 1e-4 to 2e-3 off the math (R3 candidate, owner ruling)"
milestone: M9
lane: functions-financial
status: blocked
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The owner rules whether XIRR roots with |root| below about 2e-5 become a decision 0188 R3 (`MATH-OVER-EXCEL`)
region, and the engine follows that ruling. Without the ruling, the engine keeps matching Excel's value bit for bit,
as it does today.

## Blocked because
Blocked because: owner ruling (decision 0188 R3: a new `MATH-OVER-EXCEL` case needs a decision record carrying an
owner ruling).

## Context
- Found by T-3353's Gate A fresh sweep (probes-numeric/xirr-two-flow-gate-a-sweep, excel-graph, 2026-10-31,
  stratum near-zero, rows 91-105). Excel's XIRR stops once the bisection half-width is below 5e-9 + 1e-8·|m|
  (docs/excel-semantics/financial.md "XIRR is a bracketing bisection, not Newton"). Near a zero root the absolute
  5e-9 term dominates, so the result is about 1e-9 to 3e-9 off in absolute terms. Relative to the 50-digit closed form
  that is more than G = 1e-4 on 9 of the 15 cells: rows 94 (root 1.14e-6, 1.4e-3 off), 95 (1.1e-4), 96 (2.5e-4),
  98 (2.1e-3), 99 (1.1e-3), 100 (1.4e-4), 101 (7.1e-4), 102 (1.4e-4) and 104 (3.0e-4).
- The engine matches Excel bit for bit on all 15 cells. So the floor of Gate A (within 15 digits of Excel) holds, and
  nothing fails today.
- An R3 ruling would turn this into an engine change: keep bisecting near 0 until the result is within 1e-9 relative of
  the math. It would also need `math-over-excel.tsv` entries once eval/accuracy has an XIRR table (SPEC-0029).

## Acceptance criteria
- [ ] A decision record carries the owner's ruling (R3 region or not).
- [ ] If R3: the engine converges near zero roots, a `MATH-OVER-EXCEL` entry in financial.md lists the region with
      the cells above as evidence, and every other XIRR cell still matches Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's stop rule away from zero roots (T-3353, T-3287).

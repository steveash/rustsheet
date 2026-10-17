---
id: T-1820
title: "DB and VDB bound their period loops, then lift over arrays and ranges (SPEC-0011)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-1372]
---

## Goal
DB and VDB lift every argument over arrays and multi-cell ranges, as the other core financial functions do since
T-1372, without one call being able to loop for an unbounded time.

## Context
probes/fin-argument-modes: `ROWS(DB(G25:G26,1000,6,2,7))` and `ROWS(VDB(G99:G100,300,10,0,1,2,FALSE))` are 2 in Excel,
and every `ROWS(DB|VDB(..{v;v}..))` is 2 (docs/excel-semantics/financial.md "Financial function argument modes").
T-1372 left them off the SPEC-0011 driver because their kernels loop over the periods with no cap of their own:
`db_depreciation` runs `2..period` with `period <= life + 1`, and `vdb_total_depr` steps one period at a time up to
`period`, so `DB(1,0,1E15,1E15)` or `VDB(1000,0,1E9,0,1E9)` runs for hours today, and a lifted call would multiply
that (CLAUDE.md directive 5; SPEC-0011 "Security rule for kernel work", as T-1236 found for the day loops). Bound the
loops first: a closed form where it matches the loop bit for bit on the existing evidence (ironcalc-calc DB/VDB
workbooks, probes-numeric), or a per-call cap with a DEVIATION entry. Then enter both as `Lift` in
functions/lift/kinds.rs (the `TODO(T-1820)` there), `KernelWork::Weighted` if a value-dependent loop remains.

## Acceptance criteria
- [ ] `DB` and `VDB` with a life of 1E9 or more finish in milliseconds; any capped result is a DEVIATION entry.
- [ ] The DB and VDB lift cells of probes/fin-argument-modes pass (baseline lines removed), with no new failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- DB's and VDB's base-value ulp misses (T-1374, T-1764).

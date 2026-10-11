---
id: T-1372
title: "Financial functions lift scalar arguments over arrays, and the core ones over ranges (probes/fin-argument-modes)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Every scalar argument of the M9 financial functions lifts elementwise over an array literal. The core worksheet
functions also lift over a multi-cell range, while the Analysis ToolPak ones are `#VALUE!` there, as in Excel.

## Context
probes/fin-argument-modes (excel-graph, 2026-10-11, T-1084; legend in docs/excel-semantics/financial.md, "Financial
function argument modes"). All 256 `ROWS(F(..{v;v}..))` cells are 2 in Excel. Of the 55 `ROWS(F(G1:G2,..))` cells, 18 are 2
(DB, DDB, FV, IPMT, IRR guess, ISPMT, MIRR, NPER, NPV rate, PDURATION, PMT, PPMT, PV, RATE, RRI, SLN, SYD, VDB) and 37
are `#VALUE!` (the ATP functions); the engine already matches those 37. The engine fails 274 lift cells: `#N/IMPL!` for
an array (256) and `#VALUE!` for a range (18). Reuse the lifting the math, date and lookup functions already use (search
before adding a helper; T-1290 does the same for the statistical functions). The ATP functions lift arrays but not
ranges, so the helper needs a "lift literals only" variant, or an existing way to say that. Split by family if larger than M.

## Acceptance criteria
- [ ] The lift cells of probes/fin-argument-modes pass (their baseline lines removed), and the 37 range `#VALUE!` cells
  still pass.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Base-value ulp misses (T-1374, T-0901, T-0916).

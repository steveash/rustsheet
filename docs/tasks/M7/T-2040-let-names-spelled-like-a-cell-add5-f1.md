---
id: T-2040
title: "LET names spelled like a cell (add5, f1) in the stored _xlpm. form bind like any name"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A LET name or LAMBDA parameter spelled like a cell, stored in its `_xlpm.` form, binds like any other
name: `=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),add5,mk(5),add5(1))` is 6 as in Excel, not `#VALUE!`.

## Context
Found by T-0439. probes/lambda-escaping-closures H4 (`add5`) and H5 (`f1`, `f2`) give 6 and 100001 on
excel-graph (the workbook stores the names as `_xlpm.add5`); the engine gives `#VALUE!` because it reads
`add5` and `f1` as cells (`=LET(f1,1,f1+1)` is `#VALUE!` too). H21/H22, the same formulas with names that
are no cell, pass. Probe what Excel does with such a name typed without `_xlpm.` (the API path) before
changing the A1 grammar, and record it in docs/excel-semantics/references.md.

## Acceptance criteria
- [ ] probes/lambda-escaping-closures H4 and H5 pass; a probe settles typed (unprefixed) cell-shaped names.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined names spelled like cells (unchanged).

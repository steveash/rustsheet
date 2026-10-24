---
id: T-2040
title: "LET names spelled like a cell (add5, f1) in the stored _xlpm. form bind like any name"
milestone: M7
lane: engine
status: done
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
- [x] probes/lambda-escaping-closures H4 and H5 pass; a probe settles typed (unprefixed) cell-shaped names.
- [x] `make check` green; reviewers approve.

## Out of scope
- Defined names spelled like cells (unchanged).

## Notes (first session, 2026-10-23)
- The engine already binds the stored form: `=_xlfn.LET(_xlpm.f1,1,_xlpm.f1+1)` is 2, and the H4 formula written with
  `_xlpm.` names is 6 (unit test against `Model::_set`). Only the unprefixed text, `=LET(f1,1,f1+1)`, is read as cells
  (stored back as `F1`, `#VALUE!`).
- The oracle cannot probe the unprefixed form: `tools/src/rustsheet_tools/spec2xlsx.py` writes every LET/LAMBDA name as
  `_xlpm.x` (T-0166), so excel-graph evaluated `_xlpm.add5`. The eval harness, however, hands the engine the case's
  unprefixed text. So H4/H5 fail because the harness and the oracle evaluate different formulas, not because of the
  stored-form grammar.
- Next: decide (spec/decision) between (a) the harness setting case formulas in the `_xlpm.` form spec2xlsx writes
  (matches what Excel evaluated; crates/rustsheet-eval) and (b) the typed-formula parser binding a cell-shaped name inside
  a LET/LAMBDA scope (Excel's own typed behaviour is unprobed: Excel for the web refuses such names when typed, by
  memory only, unverified). (a) needs no Excel evidence beyond the existing H4/H5.

## Outcome
- Probes (excel-graph, 2026-10-23): stored `_xlpm.` cell-shaped names bind
  (probes/let-cell-shaped-name-stored, 14 cells); typed entry of the same formulas is refused by
  Excel for the web (Graph 400 InvalidArgument), the typed control gives 2
  (probes/let-cell-shaped-name-typed-control). Recorded in references.md "LET and LAMBDA names
  spelled like a cell".
- The A1 parser binds a cell-shaped LET name or LAMBDA parameter within its call
  (`bind_reference_shaped_name`, generalizing T-1659's Lotus-dot binding); the R1C1 storage
  prefixes such names with `_xlpm.` so they reload.
- The engine still accepts the typed form Excel refuses: follow-up T-2497.
- Eval: probes known failures 1999 -> 1997 (H4, H5 fixed), 2 new cases all passing, no regressions.

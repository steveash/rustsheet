---
id: T-2497
title: "Typed LET names and LAMBDA parameters spelled like a cell are refused, as Excel refuses them"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A formula entered through the API whose LET name or LAMBDA parameter is spelled like a cell
(`=LET(f1,1,f1+1)`, `=LAMBDA(f1,f1*2)(3)`) is refused as Excel refuses it typed, while the same
formula stored with `_xlpm.` names (xlsx import) still binds.

## Context
T-2040 found that Excel for the web refuses these typed formulas (Graph range PATCH 400
InvalidArgument, 2026-10-23, for single-cell cases shaped like probes/let-cell-shaped-name-typed-control
with `{"entered": ...}` set to `=LET(f1,1,f1+1)`,
`=LET(mk,LAMBDA(a,LAMBDA(b,a+b)),add5,mk(5),add5(1))` and `=LAMBDA(f1,f1*2)(3)`; refused, so not
committed; re-run them to reproduce. The control, `=LET(x,1,x+1)`, gives 2), and binds the stored form
(probes/let-cell-shaped-name-stored). The engine's typed parser reads the typed text as its
stored form (`bind_reference_shaped_name`, parser/operands.rs), because the eval harness hands the
engine case formulas unprefixed while spec2xlsx stores them with `_xlpm.`. So refusing the typed
form first needs the harness to set case formulas in the form Excel evaluated (the `_xlpm.` names
spec2xlsx `prefix_bound_names` writes), or an API-only refusal (`Input::Formula`/`Parse`) that the
harness path does not take. Decide which in a short spec or decision record; T-1856 (names Excel
refuses at load) is the same kind of gap. docs/excel-semantics/references.md "LET and LAMBDA names
spelled like a cell".

## Acceptance criteria
- [ ] `Input::Formula("=LET(f1,1,f1+1)")` through the API is refused (an `EditError` or the
      repo's refused-formula result), probes/let-cell-shaped-name-stored and
      lambda-escaping-closures H4, H5 still pass.
- [ ] `make check` green; reviewers approve.

Also probe (stored form): a bound cell-shaped name as a range end (`LET(f1,1,f1:G2)`), a use
before its binding (`LET(x,F1,F1,2,x+F1)`), and `LET($F$1,...)`; the engine's answers for these are
unprobed hand expectations (test_let_cell_shaped_names.rs).

## Out of scope
- Lotus-dot names (`LET(R1.C1,...)`): typed entry unprobed.

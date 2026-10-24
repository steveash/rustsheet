---
id: T-2057
title: "A LET name shadows a defined name of the same name in any case"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
With a workbook defined name `Mk`, `=LET(mk,LAMBDA(a,a+1),mk(1))` reads the LET name (2), as LET and LAMBDA
names shadow outer names elsewhere, instead of `#VALUE!`.

## Context
Found by the Excel-fidelity review of T-0439 (on main too): a LET name that matches a defined name
ignoring case resolves to the defined name. Probe first (`excel-probe`): a defined name `Mk` (a LAMBDA
and a plain value) with `LET(mk,..,mk(1))`, `LET(Mk,..,Mk+1)` and a LAMBDA parameter of the same name;
record the rule in docs/excel-semantics/references.md, then fix name resolution in the parser/evaluator.

## Acceptance criteria
- [x] Probe case with oracle values; the engine matches it; unit tests.
- [x] `make check` green; reviewers approve.

## Out of scope
- Defined names spelled like cells.

## Outcome
Probe `probes/let-name-shadows-defined-name` (excel-graph, 2026-10-23): a LET name or LAMBDA parameter shadows a
defined name of the same name in any case. The parser now declares the variable instead of resolving the defined
name and skips the defined-name lookup for bound names (`bind_reference_shaped_name`, `name_operand`); unit test
in `test_fn_let.rs`; entry in docs/excel-semantics/references.md. Eval: 15 of the 16 cells of the new probe case pass; B2 `LET(Pv,LAMBDA(a,Pv),Pv(1))` (`#NAME?` in Excel, `#ERROR!` here) is a baselined known failure, T-2558.

## Handoff
Escalated per work-loop step 5b: the Excel-fidelity reviewer requested changes twice (round 1: shadowing lost
after a save and reload because stored formulas parse in R1C1 mode; round 2: the semantics entry omitted the
own-value rule B1 and the B2 oracle cell was dropped). Both are fixed and pushed; `make check` was green
before the last data-only commit, and `eval-check`, `hygiene`, `typos` and `fmt-check` are green after it.
Security, architecture and quality approved the first diff. Next: re-run `reviewer-excel-fidelity` (round 3)
on the current head, set `status: done`, open the PR, merge under the steward policy. Open follow-up: T-2558
(B2 `LET(Pv,LAMBDA(a,Pv),Pv(1))` is `#NAME?` in Excel, `#ERROR!` here; baselined known failure).

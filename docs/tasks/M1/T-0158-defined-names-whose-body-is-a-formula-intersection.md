---
id: T-0158
title: "Defined names whose body is a formula (intersection, constant) evaluate as Excel"
milestone: M1
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A defined name whose body is not a plain cell/range reference or a LAMBDA (an intersection such as
`Sheet1!$A$1:$A$3 Sheet1!$A$2:$B$2`, a constant such as `0.5`) evaluates as Excel does: an
intersection body is a reference (`ISREF` TRUE, usable by `ROWS`, `SUM`, `INDEX`, further
intersection), a disjoint one is `#NULL!`, a constant is its value.

## Context
`probes/defined-name-intersection.case.json` (oracle: excel-graph, T-0043) fails on 9 of its
10 formula cells (H9 `=ISREF(disj)` passes only because `ISREF(#NAME?)` is FALSE too): the engine answers `#NAME?` with "Defined name ... is not a reference"
(`crates/rustsheet-engine/src/model.rs` ~725). `parse_defined_names` (`new_empty.rs`) only keeps
reference and LAMBDA bodies, so any other body is dropped. Semantics:
docs/excel-semantics/references.md "A defined name may be an intersection" and "A defined name may be a constant". T-0152 also touches
name-body evaluation (`Sheet1!#REF!` bodies); reuse whatever it lands rather than adding a
second path. Recursion through names (`a` = `=b`, `b` = `=a`) must stay bounded (directive 5).

## Acceptance criteria
- [x] Every cell of `probes/defined-name-intersection.case.json` passes; baseline updated.
- [x] A self-referencing name chain does not recurse unboundedly (unit test).
- [x] `make check` green; reviewers approve.

## Out of scope
- Sheet-scoped names in case specs.

## Outcome
- Name bodies that are neither a reference, a LAMBDA nor an error literal are kept as
  `ParsedDefinedName::Formula` and evaluated where the name is used. ISREF looks through them.
  Nested name bodies are capped at 64 deep, so name cycles are `#NAME?` as in Excel
  (DEVIATION for longer chains of distinct names: references.md "A defined name may be a formula").
- New probe probes/defined-name-formula-bodies (17 cells, excel-graph): all pass.
- Eval: probes 9 fixed (all 9 failing cells of defined-name-intersection), 0 regressions.

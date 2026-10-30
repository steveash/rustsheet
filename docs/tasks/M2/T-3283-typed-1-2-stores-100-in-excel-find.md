---
id: T-3283
title: "Typed =1**2 stores =100 in Excel: find and match Excel's reading of ** in a typed formula"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A formula typed with `**` reads as Excel reads it: `=1**2` typed stores `=100` (value 100) in Excel.

## Context
probes/t3213-typed-parse-errors S!A5, B5, C5 (excel-graph): `=1**2` typed is not refused; FORMULATEXT
is `=100`. RustSheet refuses it as a parse error since T-3213 (docs/excel-semantics/syntax.md "A typed
formula that does not parse is refused"). Excel's rule is unknown: probe first (`=2**3`, `=1**`,
`=A1**2`, `=1***2`, `=1**2` stored) to tell an autocorrect of `**` from a number-literal reading
(`1E2`-like) before implementing. Code: `Model::parse_cell_formula` (model.rs), typed_input.rs.

## Acceptance criteria
- [ ] t3213-typed-parse-errors A5/B5/C5 pass, and a probe of more `**` shapes agrees.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

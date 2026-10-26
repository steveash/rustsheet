---
id: T-2795
title: "Name the fields of ParsedFormula (evaluated, static_result, written)"
milestone: M2
lane: engine
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
`ParsedFormula` is a struct with named fields (`evaluated`, `static_result`, `written`) and accessors, so no
call site reads the evaluated or the written tree by position. Behaviour does not change.

## Context
T-2781 (decision 0127) made `formula_cells::ParsedFormula` a positional 3-tuple: `.0` the stripped tree evaluation
reads, `.2` the written tree printers read (`written_formula`). Call sites in the engine (`model.rs`, `formula_cells.rs`,
`functions/database.rs`, `functions/subtotal.rs`, `legacy_formula.rs`, `actions.rs`) and in
`crates/rustsheet-xlsx/src/export/worksheets.rs` read it as `.0` or destructure it. Architecture review of T-2781
(minor finding) asked for names before more code depends on the positions.

## Acceptance criteria
- [ ] `ParsedFormula` is a struct; `stored_formula` builds it; printing sites use a `written()` accessor and
      evaluation sites an `evaluated()` one (or the fields by name).
- [ ] No eval change (`make eval-check` clean).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing what the export writes (T-2786).

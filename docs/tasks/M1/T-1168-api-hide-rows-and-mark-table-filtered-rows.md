---
id: T-1168
title: "API: hide rows and mark table-filtered rows in an edit transaction"
milestone: M1
lane: api
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
An API v0 edit transaction can hide rows and mark rows as hidden by a table filter. The eval
harness can then load probe cases with hidden and filtered rows (T-1036).

## Context
T-1036 needs `rustsheet-case/v1` to hide rows and filter tables, both in spec2xlsx and in the
engine loader (`evaluate_spec` in crates/rustsheet-eval/src/engine.rs). That loader only goes
through the `rustsheet` facade transaction (`tx.set`, `tx.add_sheet`, `tx.define_name`), and
the facade has no row visibility at all. The engine has `Model::set_row_hidden`
(crates/rustsheet-engine/src/model.rs). AGGREGATE reads `row.hidden` (functions/aggregate.rs),
but nothing in the engine tells a filtered row from a manually hidden one, and SUBTOTAL 1-11 versus
101-111 depends on that difference in Excel. Follow docs/specs for API v0 (the transaction and its
error types); this is an additive method.

## Acceptance criteria
- [ ] The transaction can hide or show a row, with a unit test and an error for an invalid row.
- [ ] Rows hidden by a filter are distinguishable in the model, or a short design note in the task
      Outcome says why that waits for T-1036's probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The case format and the AGGREGATE/SUBTOTAL probe (T-1036).

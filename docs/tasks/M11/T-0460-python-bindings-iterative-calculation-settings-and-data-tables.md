---
id: T-0460
title: "Python bindings: iterative calculation settings and data tables"
milestone: M11
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`import rustsheet` can turn iterative calculation on and off with its count and delta, and can add
a what-if data table, mirroring the Rust facade.

## Context
T-0447 added `Workbook::set_iteration` / `Iteration` and T-0441 added `Txn::add_data_table` to
crates/rustsheet. The architecture review asked for binding mirrors; bindings/python has neither.

## Acceptance criteria
- [ ] Binding methods for both, with typed errors, `.pyi` stubs and docs.
- [ ] Python tests: an iterating cycle converges, and a column data table fills its results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Removing data tables from Python.

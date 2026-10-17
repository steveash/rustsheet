---
id: T-1776
title: "Check the SPEC-0011 argument-kind table against the eval catalog; catalog marks Analysis ToolPak parameters lift: arrays"
milestone: M9
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-1770]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A test keeps the SPEC-0011 argument-kind table and eval/catalog/functions.yaml in agreement. The catalog marks the Analysis ToolPak parameters `lift: arrays`, so the generator stops passing them ranges that are always `#VALUE!`. A metamorphic test checks that every table entry lifts like its scalar call.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "The argument-kind table" and "Test plan".
- The engine exposes `#[doc(hidden)] pub fn lift_kinds(name: &str) -> Option<Vec<&'static str>>`. A test in crates/rustsheet-eval (which already parses the catalog, src/catalog.rs) checks every function with both an entry and catalog `params`:
  - `mode: value` is `Lift`, or `LiftArrays` exactly when the catalog says `lift: arrays`.
  - `array`, `aggregate`, `aggregate_a`, `range`, `criteria` and `lambda` are `Whole` or `WholeByCaller`.
  - The parameter counts agree.
- Document `lift: arrays` in the catalog header, teach tools/ (the generator and tools/tests/test_catalog.py) to accept it, and run `cd tools && uv run rustsheet-catalog-view`.
- Metamorphic test: for each table entry, build a valid scalar call from the catalog domains, then the same call with one lifted argument written `{v;v}`. Both elements must equal the scalar result.

## Acceptance criteria
- [ ] The cross-check and metamorphic tests run in `make check`. They fail on a seeded mismatch, which the PR shows once.
- [ ] Every `LiftArrays` parameter in the table carries `lift: arrays` in the catalog. The generator never passes a multi-cell range to such a parameter (a tools test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding functions to the table (the family tasks).

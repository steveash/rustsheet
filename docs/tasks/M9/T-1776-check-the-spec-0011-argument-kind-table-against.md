---
id: T-1776
title: "Derive the catalog's lift: arrays from the SPEC-0011 argument-kind table and check it, the kernel-work list and the lifts"
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
The eval catalog's `lift: arrays` key is derived from the SPEC-0011 argument-kind table and checked against it. The generator stops passing ranges to parameters that refuse them. A check makes value-dependent kernels declare their work, and a metamorphic test checks that every table entry lifts like its scalar call.

## Context
Spec: docs/specs/SPEC-0011-scalar-argument-lifting.md, "The argument-kind table", "Security rule for kernel work" and Test plan.
- The engine exposes a hidden testing hook (not public API, no stability promise): `#[doc(hidden)] pub fn lift_kinds(name: &str) -> Option<Vec<&'static str>>`.
- A command (`rustsheet-eval catalog-lift --write`, or a tools script calling it) writes `lift: arrays` into eval/catalog/functions.yaml from the table. A test fails when the committed catalog differs, and also checks the spec's one mapping rule:
  - `Lift` is `mode: value`;
  - `LiftArrays` is `mode: value` + `lift: arrays`;
  - `LiftArraysRangeWhole` is `mode: criteria` or `range` + `lift: arrays` (T-1389's D-function criteria);
  - `Whole` and `WholeByCaller` are `array`, `aggregate`, `aggregate_a`, `range`, `criteria` or `lambda` without `lift`;
  - the parameter counts agree.
- Document `lift: arrays` in the catalog header, teach tools/ (the generator and tools/tests/test_catalog.py) to accept it, and run `cd tools && uv run rustsheet-catalog-view`.
- `KernelWork` check: a list of value-dependent functions lives in the test (BINOM.DIST, POISSON, COMBIN, FACT, MULTINOMIAL, RATE/IRR/YIELD and the other solvers, the Bessel functions, …). Each one in the table must be `Weighted`, or `Constant` with a comment naming its fixed iteration cap.
- Metamorphic test: for each table entry, build a valid scalar call from the catalog domains, then the same call with one lifted argument written `{v;v}`. Both elements must equal the scalar result.

## Acceptance criteria
- [ ] The derivation check, mapping check, `KernelWork` check and metamorphic test run in `make check`. Each fails on a seeded mismatch, which the PR shows once.
- [ ] The generator never passes a multi-cell range to a `mode: value, lift: arrays` parameter (a tools test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Adding functions to the table (the family tasks).

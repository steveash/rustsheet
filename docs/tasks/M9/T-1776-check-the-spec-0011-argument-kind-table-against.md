---
id: T-1776
title: "Derive the catalog's lift: arrays from the SPEC-0011 argument-kind table and check it, the kernel-work list and the lifts"
milestone: M9
lane: harness
status: done
priority: P1
size: S
tier: opus
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
- [x] The derivation check and mapping check run in `make check`. Each fails on a seeded mismatch, which the PR shows once.
- [x] The generator never passes a multi-cell range to a `mode: value, lift: arrays` parameter (a tools test).
- [x] `make check` green; reviewers approve.

## Out of scope
- Adding functions to the table (the family tasks).
- The `KernelWork` check (split to T-1893) and the metamorphic test (split to T-1894) by the opus resume: each needs per-kernel judgement, and together they put the diff past size M.

## Handoff (resumed)
Sonnet run read the task, `functions/lift/kinds.rs` (153 `Function::` arms, `arg_kinds`) and the catalog; nothing is implemented yet. Judged beyond a sonnet run: the mapping rule touches ~150 table entries against catalog param modes (expect many mismatches needing per-function judgement), needs a new `#[doc(hidden)] lift_kinds` hook plus name->Function lookup, a `catalog-lift --write` command in rustsheet-eval, generator and tools/tests changes, a KernelWork list and a metamorphic test with seeded-mismatch demos. Next: add `lift_kinds`, write the derive/check test in rustsheet-eval first and see how many catalog entries disagree, then split the metamorphic test into its own task if the diff passes size M.

## Outcome
- The engine exposes `#[doc(hidden)] rustsheet_engine::lift_kinds_table()`. It returns every table row (name, required, params, repeat, max, work) rather than the `lift_kinds(name)` the Context sketched: the check needs counts and repeats too.
- `rustsheet-eval lift-kinds --write` writes eval/catalog/lift-kinds.json (one function per line). The Rust test `lift_kinds::committed_table_is_current` fails when the file is stale, so tools/ never builds the engine.
- `uv run rustsheet-catalog-lift [--write]` (tools/src/rustsheet_tools/catalog_lift.py) applies the spec's mapping rule (modes, `lift: arrays`, parameter counts). `--write` derived 250 `lift: arrays` keys into functions.yaml, and the modes already agreed for all 185 rows. tools/tests/test_catalog_lift.py checks the committed catalog, the idempotence of `--write`, and a seeded mismatch.
- The generator gives a `lift: arrays` param no LET/LAMBDA names, which may hold a range (tools/tests/test_generator.py).
- The KernelWork check went to T-1893 and the metamorphic test to T-1894.
- Eval delta: none (harness and catalog only).

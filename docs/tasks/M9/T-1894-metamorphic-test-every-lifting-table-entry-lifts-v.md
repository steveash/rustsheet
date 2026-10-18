---
id: T-1894
title: "Metamorphic test: every lifting-table entry lifts {v;v} like its scalar call"
milestone: M9
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-1776]
spec: docs/specs/SPEC-0011-scalar-argument-lifting.md
---

## Goal
A test proves every SPEC-0011 lifting-table entry lifts like its scalar call: with one lifted argument written `{v;v}`, both elements equal the scalar result.

## Context
Split from T-1776 (SPEC-0011 Test plan). For each row of `rustsheet_engine::lift_kinds_table()`, build a valid scalar call from the catalog domains in eval/catalog/functions.yaml (or a hand-kept table of sample arguments if reading YAML from Rust is not wanted; eval/catalog/scope.json is the harness's JSON view and could carry the domains), evaluate it, then evaluate the same call with each `Lift`/`LiftArrays` argument in turn written `{v;v}`, and compare both spilled elements to the scalar value (errors included). `WholeByCaller` arguments stay as written. The test lives in the engine's test tree (decision 0035) or rustsheet-eval.

## Acceptance criteria
- [ ] The test covers every table row and every lifted parameter, and runs in `make check`.
- [ ] It fails on a seeded wrong kind (shown once in the PR); any real mismatch it finds gets a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
-

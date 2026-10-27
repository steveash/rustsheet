---
id: T-2895
title: "Fuzz target bitcode_workbook: decoded bitcode through Model::from_bytes never panics"
milestone: M12
lane: harness
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-2913]
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
A cargo-fuzz target `bitcode_workbook` decodes arbitrary bytes as a `Workbook` through `Model::from_bytes` and, when that
loads, runs `evaluate()` and a few reads and edits, with no panic.

## Context
SPEC-0026 decision 5. Directive 5: decoded bitcode is untrusted. No fuzz target decodes bitcode today; the existing
targets are fuzz/fuzz_targets/{formula_eval,formula_parse,xlsx_import}.rs. Follow xlsx_import.rs's layout and
registration (fuzz/Cargo.toml). Seed the corpus with `Model::to_bytes()` of a few small workbooks (a formula, a style, a
shared formula, two sheets), since random bytes rarely decode. Depends on T-2913, which turns the known panics into `Err`.
The target also covers what the validator cannot: `bitcode::decode` allocates from lengths in the buffer before any
check runs, so a run that finds a decode-time allocation blow-up files it as its own task (bitcode's own limits, or a
size cap on the input).

## Acceptance criteria
- [ ] `fuzz/fuzz_targets/bitcode_workbook.rs` exists and is registered, with a seed corpus of engine-written workbooks.
- [ ] A local run of at least 10 minutes finds no crash, or each crash found is filed as a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- CI scheduling of long fuzz runs (M12 release work).

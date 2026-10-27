---
id: T-2897
title: "Python: drop the bitcode and icalc loaders, apply_external_diffs and test_panic (SPEC-0026)"
milestone: M12
lane: api
status: todo
priority: P1
size: S
tier: sonnet
depends_on: []
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
The Python package no longer accepts bitcode or icalc bytes and no longer ships a panic function. Removed: `load_from_bytes`,
`load_from_icalc`, `create_user_model_from_bytes`, `create_user_model_from_icalc`, `UserModel.from_bytes`,
`UserModel.from_icalc`, `UserModel.apply_external_diffs`, `UserModel.flush_send_queue`, the module function
`test_panic` and `Model.test_panic`. The writers that have no reader left go too: `save_to_icalc` and `to_bytes`
on both classes.

## Context
SPEC-0026 decisions 1, 2 and 5 (docs/specs/SPEC-0026-python-legacy-surface.md, its symbol table).
Code: bindings/python/src/lib.rs, raw_model.rs, user_model.rs; bindings/python/python/rustsheet/__init__.py (imports and
`__all__`) and _ironcalc.pyi. Tests to drop or rewrite: tests/test_io.py (`test_icalc_roundtrip` and the bytes
round-trip; keep the xlsx ones), tests/test_diffs.py (the whole file) and docs/examples/collaboration.py (delete it). No test calls `test_panic`. Pre-v1, so this ships with no warning release
(SPEC-0026 decision 5); note it in the package changelog or README.

## Acceptance criteria
- [ ] None of the listed symbols is importable from `rustsheet` (a test asserts it for each name).
- [ ] `grep -rn "test_panic\|from_icalc\|from_bytes\|apply_external_diffs" bindings/python` finds only that test and
      the facade's `Workbook.from_bytes` (xlsx bytes, which stays).
- [ ] `make check` green (python job included); reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving the classes to `rustsheet.legacy` (T-2898).
- The engine's own `Model::from_bytes` and `apply_external_diffs` (T-2894, T-2896).

---
id: T-2896
title: "UserModel::apply_external_diffs refuses crafted diffs instead of panicking"
milestone: M12
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-2913]
spec: docs/specs/SPEC-0026-python-legacy-surface.md
---

## Goal
`UserModel::apply_external_diffs` returns `Err` for crafted `QueueDiffs` bytes (dangling sheet, style or row/column
indices, out-of-range cells) and leaves the model unchanged, instead of panicking.

## Context
SPEC-0026 decision 4: diff bytes come from a peer, so they are untrusted under directive 5.
`UserModel::apply_external_diffs` (crates/rustsheet-engine/src/user_model/common.rs) decodes `Vec<QueueDiffs>` with
bitcode and applies each `Diff` as it is. A diff that names a style index or sheet the model does not have reaches the
same unchecked indexing as T-2913's reproductions. Validate each decoded diff against the model before applying any
(reuse T-2913's index checks), so a refused list changes nothing. Write the first test the way T-2882 reproduced the
load panics: flush a real send queue, change one index in the bytes, apply. The Python binding drops this entry point
(T-2897); this task hardens the Rust API, which the web UI layer still uses.

## Acceptance criteria
- [ ] A crafted diff list with a dangling style index and one with a dangling sheet index each get `Err`, with no panic
      and the model unchanged; the existing diff tests still pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- A fuzz target for diffs (file one if this task finds more than the shapes above).

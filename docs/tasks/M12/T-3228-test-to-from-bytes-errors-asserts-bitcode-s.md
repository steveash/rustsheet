---
id: T-3228
title: "test_to_from_bytes::errors asserts bitcode's debug-only error text: fails in release builds"
milestone: M12
lane: quality
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`test::user_model::test_to_from_bytes::errors` passes in both debug and release builds, so a release test run of the
engine is green on `main`.

## Context
On `main` (8f5515462, 2026-10-30) `cargo test --release -p rustsheet_engine --lib test_to_from_bytes` fails:
`left: "Error parsing workbook: bitcode error"`, `right: "Error parsing workbook: invalid packing"`. CI passes it
because it tests a debug build.

Cause: `Model::from_bytes` (`crates/rustsheet-engine/src/model.rs:1281`) formats bitcode's error into its message
(`format!("Error parsing workbook: {e}")`). bitcode 0.6's `Display for Error` (`src/error.rs`) writes the detailed
text only under `#[cfg(debug_assertions)]`, and `"bitcode error"` otherwise. The test
(`src/test/user_model/test_to_from_bytes.rs:26`) asserts the debug-only text, so its outcome depends on the build
profile.

Prefer the first option unless a caller needs a stable message (grep for `Error parsing workbook` first):
- Assert only the build-independent part: the `Error parsing workbook: ` prefix.
- Have `from_bytes` return a fixed message of our own that doesn't depend on bitcode's text.

The second option changes a public error string, so check the Python bindings and any callers that match on it. Related work:
T-2895 (bitcode fuzz target) and T-2897 (dropping the Python bitcode loader).

## Acceptance criteria
- [ ] `cargo test --release -p rustsheet_engine --lib test_to_from_bytes` and the debug run both pass.
- [ ] No test asserts text that bitcode writes only under `debug_assertions`.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Replacing bitcode or changing the byte format.

---
id: T-1668
title: "Upgrade zip to 9.0.3: every earlier release is yanked and cargo deny fails CI"
milestone: M2
lane: quality
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
CI's `cargo deny` check passes again: the workspace depends on a `zip` release that is not yanked.

## Context
On 2026-10-16 crates.io lists every `zip` release before 9.0.3 as yanked, including the 0.6.6 that rustsheet, rustsheet-xlsx and rustsheet-eval use. deny.toml sets `yanked = "deny"`, so the hygiene job fails on `main` (CI run 38846359359) and on every PR. zip 9.0.3 needs Rust 1.88, the workspace MSRV.

## Acceptance criteria
- [x] All three crates use `zip` 9.0.3 with only the zlib-rs deflate backend feature (see the Cargo.toml files); Cargo.lock has no yanked `zip`.
- [x] xlsx import/export and eval snapshot/fetch behave as before (tests and the eval gate unchanged).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
- `zip` 0.6.6 → 9.0.3 in rustsheet, rustsheet-xlsx and rustsheet-eval, with `default-features = false` and only the zlib-rs deflate backend feature. zip 9's `deflate` feature also pulls in the zopfli compressor, which we don't need. New transitive dependencies: zlib-rs (Zlib licence, allowed), typed-path, nix (unix-only), bitflags, cfg_aliases.
- API port:
  - `SimpleFileOptions` replaces `FileOptions`.
  - `ZipFile<'a, R>` now takes a reader type.
  - Entry names are now fallible, so `PartNames::from_archive` (rustsheet-xlsx) builds the part index for both the importer and the eval snapshot.
  - In `fetch.rs`, `enclosed_name` now returns an owned path.
- Behavior: zip 9 refuses to inflate an entry past its declared size. A zip that understates a size is now rejected as `LoadError::Format` before the inflated-bytes budget (`LoadError::Options`) is reached. The test and SPEC-0001 say so. Honest oversized entries still hit the budget.
- Eval: no change (0 fixed, 0 regressions).

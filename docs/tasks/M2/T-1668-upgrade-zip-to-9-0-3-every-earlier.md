---
id: T-1668
title: "Upgrade zip to 9.0.3: every earlier release is yanked and cargo deny fails CI"
milestone: M2
lane: quality
status: in-progress
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
- [ ] All three crates use `zip` 9.0.3 with only the deflate feature; Cargo.lock has no yanked `zip`.
- [ ] xlsx import/export and eval snapshot/fetch behave as before (tests and the eval gate unchanged).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
